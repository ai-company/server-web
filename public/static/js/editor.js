// Copyright 2021, ai-company
// https://github.com/ai-company

"use strict";

function iconSet(klass, path, pattern) {
    return icon_ => {
        return `<svg class="${klass}"><use href="${path}#${pattern.replace("{}", icon_)}"/></svg>`;
    };
}

const icon = iconSet("icon", "/static/icons/teenyicons/outline.svg", "outline--{}");

function scroll_diff_into_view(id) {
    document.getElementById(`diff-${id}`).scrollIntoView({ block: "center", inline: "center", behavior: "smooth" });
}

function scroll_card_into_view(id) {
    document.getElementById(`card-${id}`).scrollIntoView({ block: "center", inline: "center", behavior: "smooth" });
}

const EditorModel = {
    // data
    corrections: [],
    loading: false,
    editing: true,
    currentStep: 0,
    activeItem: null,
    canProceed: false,
    error: null,

    // actions
    getCorrections: function (e, after) {
        const text = document.getElementById("editor-textarea").value;
        const self = this;

        this.loading = true;

        m.request({
            method: "POST",
            url: "/api/v1/correct",
            body: text,
            serialize: v => v,
        }).then(
            function (corrections) {
                self.corrections = corrections;
                self.loading = false;
                self.error = null;
                after();
            },
            function (error) {
                console.log("correction request failed", error);
                self.loading = false;
                self.error = "Orto har nogle midlertidige tekniske problemer.\nPrøv igen om lidt!";
            }
        );
    },

    _getChangeText: function (id) {
        const index = this.corrections.findIndex(i => i.index == id);
        const item = this.corrections[index];
        const origin = Array.isArray(item.origin) ? item.origin.join(" ") : item.origin;
        const change = Array.isArray(item.change)
            ? item.change.map(i => i.change || i.origin).join("")
            : item.change || item.origin;

        return [index, item, origin, change];
    },

    doAccept: function (id) {
        const [index, item, origin, change] = this._getChangeText(id);
        console.log("doAccept", id, index);

        if (item.type == "remove") {
            this.corrections.splice(index, 1);
        } else {
            this.corrections.splice(index, 1, { type: "none", origin: change });
        }
    },
    doIgnore: function (id) {
        const [index, item, origin, change] = this._getChangeText(id);
        console.log("doIgnore", id, index);

        if (item.type == "add") {
            this.corrections.splice(index, 1);
        } else {
            this.corrections.splice(index, 1, { type: "none", origin: origin });
        }
    },
    doReport: function (id) {
        const data = `diff=${encodeURIComponent(JSON.stringify(EditorModel.corrections))}&ident=${id}`;

        const xhr = new XMLHttpRequest();
        xhr.open("POST", "/report");
        xhr.setRequestHeader("content-type", "application/x-www-form-urlencoded");
        xhr.send(data);

        this.doIgnore(id);
    },

    doAcceptAll: function (e) {
        this.corrections.filter(i => i.type != "none" && i.type != "space").map(item => this.doAccept(item.index));
    },

    doIgnoreAll: function (e) {
        this.corrections.filter(i => i.type != "none" && i.type != "space").map(item => this.doIgnore(item.index));
    },
};

const Orto = {
    view: function (self) {
        return [m(Editor), m(CorrectionPanel)];
    },
};

const Editor = {
    oncreate: function (self) {
        self.dom.focus();
    },
    view: function (self) {
        return EditorModel.editing
            ? m("textarea#editor-textarea", {
                  placeholder: "Skriv eller indsæt din tekst her…",
                  oninput: e => {
                      EditorModel.canProceed = Steps[EditorModel.currentStep].canProceed() === true;
                  },
              })
            : m(
                  "#editor-textarea",
                  EditorModel.corrections.map(i =>
                      i.type == "none" || i.type == "space"
                          ? i.origin
                          : m(
                                "span.change",
                                {
                                    id: `diff-${i.index}`,
                                    class: EditorModel.activeItem == i.index ? "active" : "",
                                    onmousedown: () => {
                                        EditorModel.activeItem = i.index;
                                        scroll_card_into_view(i.index);
                                    },
                                },
                                m(DiffTypeMap[i.type], { key: i.index, item: i })
                            )
                  )
              );
    },
};

// { "index": int, "type": "add", "change": str, "explain": str? }
const DiffAdd = {
    view: function (self) {
        return m("ins", self.attrs.item.change);
    },
};

// { "index": int, "type": "remove", "origin": str, "explain": str? }
const DiffRemove = {
    view: function (self) {
        return m("del", self.attrs.item.origin);
    },
};

// { "index": int, "type": "replace", "origin": str, "change": str, "explain": str? }
const DiffReplace = {
    view: function (self) {
        return m("span.rep", self.attrs.item.change);
    },
};

// { "index": int, "type": "split", "origin": str, "change": Change<Type>[], "explain": str? }
const DiffSplit = {
    view: function (self) {
        return m("ins", `${self.attrs.item.change.map(item => item.change || item.origin).join("")}`);
    },
};

// { "index": int, "type": "merge", "origin": str[], "change": str, "explain": str? }
const DiffMerge = {
    view: function (self) {
        const origin = self.attrs.item.change;
        return m("ins", origin);
    },
};

const DiffTypeMap = {
    add: DiffAdd,
    remove: DiffRemove,
    replace: DiffReplace,
    split: DiffSplit,
    merge: DiffMerge,
};

const CorrectionPanel = {
    view: function (self) {
        const currentStep = Steps[EditorModel.currentStep];
        EditorModel.canProceed = currentStep.canProceed() === true;

        return m("aside#corrections", [
            m(".corrections-header.step-title", [
                m("h2.corrections-title", currentStep.title),
                m(
                    "ol.corrections-progress.progress",
                    Steps.map((_, i) =>
                        i < EditorModel.currentStep
                            ? m("li.progress-complete")
                            : i == EditorModel.currentStep
                            ? m("li.progress-active")
                            : m("li")
                    )
                ),
            ]),
            m(".corrections-step", m(currentStep)),
            m(".corrections-header.corrections-step-next", [
                EditorModel.error ? m(".error.small", EditorModel.error) : null,
                m(
                    // mobile keeps the hover state in the last tap position, which makes the tooltip pop up
                    // replacing button with div while loading, is an ugly workaround that gets rid of focus and hover
                    EditorModel.loading ? ".corrections-submit.button" : "button.corrections-submit",
                    EditorModel.loading
                        ? {
                              disabled: true,
                          }
                        : EditorModel.canProceed
                        ? {
                              type: "submit",
                              onclick: e => {
                                  currentStep.proceed(
                                      e,
                                      () => (EditorModel.currentStep = (EditorModel.currentStep + 1) % Steps.length)
                                  );
                              },
                          }
                        : {
                              disabled: true,
                              "aria-label": currentStep.canProceed(),
                              "data-microtip-position": "top",
                              role: "tooltip",
                          },
                    [
                        EditorModel.loading
                            ? m("span.spinner")
                            : m(
                                  "span.load-label",
                                  EditorModel.currentStep < Steps.length - 1 ? "Tjek tekst" : "Færdiggør"
                              ),
                    ]
                ),
            ]),
        ]);
    },
};

const StepStart = {
    title: "Indsæt tekst",
    view: function (self) {
        return [
            m(".note-header", "Der er intet at tjekke endnu."),
            m(".note", "Skriv eller indsæt den tekst, du vil have tjekket igennem."),
        ];
    },
    proceed: function (e, nextStep) {
        EditorModel.getCorrections(e, () => {
            EditorModel.editing = false;
            EditorModel.activeItem = 0;
            nextStep();
        });
    },
    canProceed: function () {
        const textarea = document.getElementById("editor-textarea");
        return (textarea && textarea.value.trim().length > 0) || "Skriv eller indsæt tekst, inden du kan fortsætte.";
    },
};

const StepGrammar = {
    title: "Forslag til rettelser",
    view: function (self) {
        return EditorModel.corrections.filter(i => i.type != "none" && i.type != "space").length == 0
            ? [
                  m(".note-header", "Alt er i orden."),
                  m(".note", "Din teksts stavning, grammatik og tegnsætning er som den skal være."),
              ]
            : [
                  m(
                      ".corrections-list",
                      EditorModel.corrections
                          .filter(i => i.type != "none" && i.type != "space")
                          .map(i => m(CorrectionCard, { key: i.index, item: i }))
                  ),
                  m(".corrections-massactions", [
                      m(
                          "button.massactions-action.accept-all.accept",
                          { onclick: e => EditorModel.doAcceptAll(e) },
                          m.trust(icon("tick-circle")),
                          " Acceptér alt"
                      ),
                      m(
                          "button.massactions-action.ignore-all.ignore",
                          { onclick: e => EditorModel.doIgnoreAll(e) },
                          m.trust(icon("bin")),
                          " Ignorér alt"
                      ),
                  ]),
              ];
    },
    proceed: function (e, nextStep) {
        EditorModel.editing = true;
        m.redraw.sync();
        document.getElementById("editor-textarea").value = EditorModel.corrections.map(i => i.origin).join("");
        nextStep();
    },
    canProceed: function () {
        return (
            EditorModel.corrections.filter(i => i.type != "none" && i.type != "space").length == 0 ||
            "Acceptér eller ignorér alle forslag, inden du kan fortsætte."
        );
    },
};

const Steps = [StepStart, StepGrammar];

const CorrectionCard = {
    view: function (self) {
        const noSpaceItems = EditorModel.corrections.filter(i => i.type != "space");
        const index = noSpaceItems.findIndex(i => i.index == self.attrs.item.index);

        const left = noSpaceItems.slice(0, index);
        let ctxLeft = left[left.length - 1];
        ctxLeft = ctxLeft
            ? Array.isArray(ctxLeft.change)
                ? ctxLeft.change.map(i => i.change || i.origin).join(" ")
                : ctxLeft.change || ctxLeft.origin
            : "";

        const right = noSpaceItems.slice(index + 1);
        let ctxRight = right[0];
        ctxRight = ctxRight
            ? Array.isArray(ctxRight.change)
                ? ctxRight.change.map(i => i.change || i.origin).join(" ")
                : ctxRight.change || ctxRight.origin
            : "";

        const explanation = [
            self.attrs.item.explain,
            ...(Array.isArray(self.attrs.item.change) ? self.attrs.item.change.map(i => i.explain || null) : []),
        ].filter(i => i);

        return m(
            ".corrections-card",
            {
                id: `card-${self.attrs.item.index}`,
                class: EditorModel.activeItem == self.attrs.item.index ? "active" : "",
                onmouseenter: () => {
                    EditorModel.activeItem = self.attrs.item.index;
                    scroll_diff_into_view(self.attrs.item.index);
                },
                onmouseleave: () => {
                    EditorModel.activeItem = null;
                },
            },
            [
                m(".corrections-context", [
                    m("span.dim", `${ctxLeft} `),
                    m(ChangeTypeMap[self.attrs.item.type], { item: self.attrs.item }),
                    m("span.dim", ` ${ctxRight}`),
                ]),
                m(
                    "ul.corrections-explanation",
                    explanation.map(i => m("li", i))
                ),
                m(CorrectionCardActions, { itemId: self.attrs.item.index }),
            ]
        );
    },
};

const CorrectionCardActions = {
    view: function (self) {
        return m(".corrections-actions", [
            m(
                "button.corrections-action.accept",
                { onclick: e => EditorModel.doAccept(self.attrs.itemId), title: "Acceptér" },
                m.trust(icon("tick-circle")),
                " Acceptér"
            ),
            m(
                "button.corrections-action.ignore",
                { onclick: e => EditorModel.doIgnore(self.attrs.itemId), title: "Ignorér" },
                m.trust(icon("bin"))
            ),
            m(
                "button.corrections-action.report",
                { onclick: e => EditorModel.doReport(self.attrs.itemId), title: "Reportér" },
                m.trust(icon("flag"))
            ),
        ]);
    },
};

// { "index": int, "type": "add", "change": str, "explain": str? }
const ChangeAdd = {
    view: function (self) {
        return m("ins", self.attrs.item.change);
    },
};

// { "index": int, "type": "remove", "origin": str, "explain": str? }
const ChangeRemove = {
    view: function (self) {
        return m("del", self.attrs.item.origin);
    },
};

// { "index": int, "type": "replace", "origin": str, "change": str, "explain": str? }
const ChangeReplace = {
    view: function (self) {
        return m("span.rep", m.trust(`${self.attrs.item.origin} ${icon("arrow-right")} ${self.attrs.item.change}`));
    },
};

// { "index": int, "type": "split", "origin": str, "change": Change<Type>[], "explain": str? }
const ChangeSplit = {
    view: function (self) {
        const origin = self.attrs.item.origin;

        let left = self.attrs.item.change[0];
        left = left.change || left.origin;
        let right = self.attrs.item.change[1];
        right = right.change || right.origin;

        return [
            m("span.nowrap", [m("span.rep", origin), " ", m.trust(icon("arrow-right"))]),
            m("span.nowrap", [" ", self.attrs.item.change.map(item => m("ins", `${item.change || item.origin}`))]),
        ];
    },
};

// { "index": int, "type": "merge", "origin": str[], "change": str, "explain": str? }
const ChangeMerge = {
    view: function (self) {
        const left = self.attrs.item.origin[0];
        // left = left.change || left.origin;
        const right = self.attrs.item.origin[1];
        // right = right.change || right.origin;

        const origin = self.attrs.item.change;
        return [
            m("span.nowrap", [
                self.attrs.item.origin.map(item => [m("span.rep", `${item}`), " "]),
                " ",
                m.trust(icon("arrow-right")),
            ]),
            m("span.nowrap", [" ", m("ins", origin)]),
        ];
    },
};

const ChangeTypeMap = {
    add: ChangeAdd,
    remove: ChangeRemove,
    replace: ChangeReplace,
    split: ChangeSplit,
    merge: ChangeMerge,
};
