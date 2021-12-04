use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, JsonRender, Output, RenderContext,
    RenderError,
};

use std::{
    collections::HashMap,
    error::Error,
    fmt::{Debug, Display},
    fs::File,
    io::Read,
    ops::Deref,
    pin::Pin,
    str::from_utf8,
};

use actix_multipart::Multipart;
use actix_web::{
    dev,
    error::ErrorBadRequest,
    http::Method,
    web::{Form, Query},
    FromRequest, HttpMessage, HttpRequest,
};

use serde::{de::DeserializeOwned, Deserialize, Serialize};

use futures::{stream::StreamExt, Future, FutureExt};

use crate::route::{self};

/// Handlebars asset helper
///
/// ```
/// {{link "preload" type="..." as="..." src="..."}}
/// {{link "cold|hot" type="stylesheet|script" src="..."}}
/// ```
///
/// * Cold cache resets every minor version change.
/// * Hot cache resets every patch version change.
/// * Both caches reset on major version change.
#[derive(Clone, Copy)]
pub struct AssetHelper {
    pub v_major: usize,
    pub v_minor: usize,
    pub v_patch: usize,
}

impl HelperDef for AssetHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _r: &'reg Handlebars<'reg>,
        _ctx: &'rc Context,
        _rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let asset_type = h
            .param(0)
            .ok_or(RenderError::new(
                "asset: specify type: preload | hot | cold",
            ))?
            .value()
            .render();

        let node = match asset_type.as_str() {
            "preload" => {
                let type_ = h
                    .hash()
                    .get("type")
                    .ok_or(RenderError::new(
                        "asset: preload: specify 'type' - mime type of asset",
                    ))?
                    .value()
                    .render();

                let as_ = h
                    .hash()
                    .get("as")
                    .ok_or(RenderError::new(
                        "asset: preload: specify 'as' - html link 'as'",
                    ))?
                    .value()
                    .render();

                let src = h
                    .hash()
                    .get("src")
                    .ok_or(RenderError::new("asset: preload: specify 'src'"))?
                    .value()
                    .render();

                self.preload(&type_, &as_, &src)
            }

            "cold" | "hot" => {
                let type_ = h
                    .hash()
                    .get("type")
                    .ok_or(RenderError::new(
                        "asset: hot|cold: specify 'type' - stylesheet | script",
                    ))?
                    .value()
                    .render();

                let src = h
                    .hash()
                    .get("src")
                    .ok_or(RenderError::new("asset: hot|cold: specify 'src'"))?
                    .value()
                    .render();

                match asset_type.as_str() {
                    "cold" => self.cold(&type_, &src),
                    "hot" => self.hot(&type_, &src),
                    _ => unreachable!(),
                }
            }

            _ => {
                return Err(RenderError::new(
                    "asset: specify type: preload | hot | cold",
                ))
            }
        };

        out.write(&node)
            .map_err(|_| RenderError::new("asset: failed to write"))
    }
}

impl AssetHelper {
    /// Preload asset, is not versioned
    pub fn preload(&self, type_: &str, as_: &str, src: &str) -> String {
        format!(
            "<link rel='preload' type='{}' as='{}' href='{}' crossorigin='anonymous'>",
            type_, as_, src
        )
    }

    /// Cold asset, resets for every Minor version
    pub fn cold(&self, type_: &str, src: &str) -> String {
        match type_ {
            "script" => format!(
                "<script src='{}?v={}.{}'></script>",
                src, self.v_major, self.v_minor
            ),
            _ => format!(
                "<link rel='{}' href='{}?v={}.{}' />",
                type_, src, self.v_major, self.v_minor
            ),
        }
    }

    /// Hot asset, resets for every Patch version
    pub fn hot(&self, type_: &str, src: &str) -> String {
        match type_ {
            "script" => format!(
                "<script src='{}?v={}.{}.{}'></script>",
                src, self.v_major, self.v_minor, self.v_patch
            ),
            _ => format!(
                "<link rel='{}' href='{}?v={}.{}.{}' />",
                type_, src, self.v_major, self.v_minor, self.v_patch
            ),
        }
    }
}

/// Handlebars embed helper
///
/// ```
/// {{embed src="..."}}
/// {{embed type="stylesheet|script" src="..."}}
/// ```
#[derive(Clone, Copy)]
pub struct EmbedHelper;

impl HelperDef for EmbedHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _r: &'reg Handlebars<'reg>,
        _ctx: &'rc Context,
        _rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let failure_msg = RenderError::new("embed: failed to write");

        let type_ = h.hash().get("type");

        let src = h
            .hash()
            .get("src")
            .ok_or(RenderError::new("embed: specify 'src'"))?
            .value()
            .render();

        let body = {
            let mut file = File::open(&src).map_err(|e| {
                RenderError::new(format!(
                    "embed: failed to open file {}: {}",
                    src,
                    e.to_string()
                ))
            })?;

            let mut buf = Vec::new();
            file.read_to_end(&mut buf)?;

            String::from_utf8(buf)
                .map_err(|_| RenderError::new(format!("embed: file {} is not valid utf8", src)))?
        };

        if let Some(type_) = type_ {
            match type_.value().render().as_str() {
                "stylesheet" => out.write(&self.stylesheet(&src, &body)).map_err(|_| failure_msg),
                "script" => out.write(&self.script(&src, &body)).map_err(|_| failure_msg),
                t => Err(RenderError::new(
                    format!("embed: unknown type `{}`, expected: stylesheet | script. Omit type for plain embed", t)
                ))
            }
        } else {
            out.write(&self.plain(&src, &body)).map_err(|_| failure_msg)
        }
    }
}

impl EmbedHelper {
    pub fn stylesheet(&self, src: &str, body: &str) -> String {
        format!("<style data-src='{}'>\n{}\n</style>", src, body)
    }

    pub fn script(&self, src: &str, body: &str) -> String {
        format!(
            "<script data-src='{}'>\n{}\n</script>",
            src,
            body.replace("</script>", "&lt;/script&gt;")
        )
    }

    pub fn plain(&self, src: &str, body: &str) -> String {
        format!("<!-- \\/ {} \\/ -->\n{}\n<!-- /\\ {0} /\\ -->", src, body)
    }
}

#[derive(Clone, Copy)]
pub struct UrlHelper;

impl HelperDef for UrlHelper {
    fn call<'reg: 'rc, 'rc>(
        &self,
        h: &Helper<'reg, 'rc>,
        _r: &'reg Handlebars<'reg>,
        _ctx: &'rc Context,
        _rc: &mut RenderContext<'reg, 'rc>,
        out: &mut dyn Output,
    ) -> HelperResult {
        let path = h
            .param(0)
            .ok_or(RenderError::new("url: path not specified"))?
            .value()
            .render();

        out.write(&route::url(&path))
            .map_err(|_| RenderError::new("url: failed to write"))
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum FormField {
    Binary(Vec<u8>),
    String(String),
}

impl FormField {
    pub fn binary(&self) -> &[u8] {
        match self {
            Self::Binary(f) => f,
            Self::String(s) => s.as_bytes(),
        }
    }

    pub fn string(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    pub fn is_file(&self) -> bool {
        match self {
            Self::Binary(_) => true,
            _ => false,
        }
    }

    pub fn is_string(self) -> bool {
        !self.is_file()
    }
}

#[derive(Debug)]
pub struct FormData(pub HashMap<String, FormField>);

impl FormData {
    pub fn into_inner(self) -> HashMap<String, FormField> {
        self.0
    }

    pub fn parse<T: DeserializeOwned + Debug>(&self) -> Result<T, RemapError> {
        serde_remap(&self.0)
    }
}

impl AsRef<HashMap<String, FormField>> for FormData {
    fn as_ref(&self) -> &HashMap<String, FormField> {
        &self.0
    }
}

impl From<&FormData> for HashMap<String, Vec<u8>> {
    fn from(data: &FormData) -> HashMap<String, Vec<u8>> {
        data.0
            .iter()
            .map(|(k, v)| (k.to_owned(), v.binary().to_owned()))
            .collect()
    }
}

impl Deref for FormData {
    type Target = HashMap<String, FormField>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl FromRequest for FormData {
    type Error = actix_web::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;
    type Config = ();

    fn from_request(_req: &HttpRequest, _payload: &mut dev::Payload) -> Self::Future {
        let content_type = _req.content_type();

        if _req.method() == Method::GET {
            let form = Query::<HashMap<String, FormField>>::from_request(_req, _payload);

            async move {
                match form.await {
                    Ok(data) => Ok(FormData(
                        data.iter()
                            .filter_map(|(k, v)| {
                                v.string().and_then(|v| {
                                    if !v.trim().is_empty() {
                                        Some((k.into(), FormField::String(v.into())))
                                    } else {
                                        None
                                    }
                                })
                            })
                            .collect(),
                    )),
                    Err(e) => {
                        let cause = format!("could not parse form: {}", e.to_string());
                        Err(ErrorBadRequest(cause))
                    }
                }
            }
            .boxed_local()
        } else if content_type == "application/x-www-form-urlencoded" {
            let form = Form::<HashMap<String, FormField>>::from_request(_req, _payload);

            async move {
                match form.await {
                    Ok(data) => Ok(FormData(
                        data.iter()
                            .filter_map(|(k, v)| {
                                v.string().and_then(|v| {
                                    if !v.trim().is_empty() {
                                        Some((k.into(), FormField::String(v.into())))
                                    } else {
                                        None
                                    }
                                })
                            })
                            .collect(),
                    )),
                    Err(e) => {
                        let cause = format!("could not parse form: {}", e.to_string());
                        Err(ErrorBadRequest(cause))
                    }
                }
            }
            .boxed_local()
        } else if content_type == "multipart/form-data" {
            let form = Multipart::from_request(_req, _payload);

            async move {
                let mut form = form.await.unwrap();

                let mut items = HashMap::new();

                while let Some(item) = form.next().await {
                    let field = item.unwrap();
                    let content_disposition = field.content_disposition().unwrap();

                    let name = content_disposition.get_name().unwrap();
                    let is_file = content_disposition.get_filename().is_some();
                    let chunks: Vec<_> = field.collect().await;
                    let mut data = Vec::new();

                    for chunk in chunks {
                        data.extend(chunk.unwrap());
                    }

                    if let (Ok(data), false) = (from_utf8(&data), is_file) {
                        if !data.trim().is_empty() {
                            items.insert(name.to_owned(), FormField::String(data.to_owned()));
                        }
                    } else if !data.is_empty() {
                        items.insert(name.to_owned(), FormField::Binary(data));
                    }
                }

                Ok(FormData(items))
            }
            .boxed_local()
        } else {
            async move { Err(ErrorBadRequest("invalid content type".to_owned())) }.boxed_local()
        }
    }
}

#[derive(Debug)]
pub enum RemapError {
    SerializationError(Box<dyn Error>),
    DeserializationError(Box<dyn Error>),
}

impl Display for RemapError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SerializationError(e) => write!(f, "Serialization error: {}", e)?,
            Self::DeserializationError(e) => write!(f, "Deserialization error: {}", e)?,
        }
        Ok(())
    }
}

pub fn serde_remap<S: Serialize, D: DeserializeOwned + Debug>(input: S) -> Result<D, RemapError> {
    serde_json::from_str(
        &serde_json::to_string(&input).map_err(|e| RemapError::SerializationError(Box::new(e)))?,
    )
    .map_err(|e| RemapError::DeserializationError(Box::new(e)))
}
