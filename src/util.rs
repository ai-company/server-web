use handlebars::{
    Context, Handlebars, Helper, HelperDef, HelperResult, JsonRender, Output, RenderContext,
    RenderError,
};

/// Handlebars asset helper
///
/// ```
/// {{asset "preload" type="..." as="..." src="..."}}
/// {{asset "cold|hot" type="stylesheet|script" src="..."}}
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
        h: &Helper,
        _: &Handlebars,
        _: &Context,
        _rc: &mut RenderContext,
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

            _ => Err(RenderError::new(
                "asset: specify type: preload | hot | cold",
            ))?,
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
                "<script src='{}?v{}.{}'></script>",
                src, self.v_major, self.v_minor
            ),
            _ => format!(
                "<link rel='{}' href='{}?v{}.{}' />",
                type_, src, self.v_major, self.v_minor
            ),
        }
    }

    /// Hot asset, resets for every Patch version
    pub fn hot(&self, type_: &str, src: &str) -> String {
        match type_ {
            "script" => format!(
                "<script src='{}?v{}.{}.{}'></script>",
                src, self.v_major, self.v_minor, self.v_patch
            ),
            _ => format!(
                "<link rel='{}' href='{}?v{}.{}.{}' />",
                type_, src, self.v_major, self.v_minor, self.v_patch
            ),
        }
    }
}
