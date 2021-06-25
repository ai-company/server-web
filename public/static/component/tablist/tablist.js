const MTabs = {
    instances: 0,

    oninit: function (vnode) {
        this.instance = MTabs.instances;
        this.active_tab = 0;
        this.focused_tab = 0;
        this.total_tabs = vnode.attrs.tabs.header.length;

        MTabs.instances++;
    },

    view: function (vnode) {
        return m(
            ".mtabs[role=tablist]",
            {
                onkeydown: e => {
                    if (e.keyCode === 39 || e.keyCode === 37) {
                        if (e.keyCode === 39) {
                            // Move right
                            this.focused_tab++;

                            // If at the end, move to the start
                            if (this.focused_tab >= this.total_tabs) {
                                this.focused_tab = 0;
                            }
                        } else if (e.keyCode === 37) {
                            // Move left
                            this.focused_tab--;

                            // If at the start, move to the end
                            if (this.focused_tab < 0) {
                                this.focused_tab = this.total_tabs - 1;
                            }
                        }

                        document.getElementById(`tab-${this.instance}-${this.focused_tab}`).focus();
                    }
                },
            },
            [
                m(
                    ".tabs-head",
                    vnode.attrs.tabs.header.map((tab, i) =>
                        m(
                            "button",
                            {
                                key: i,
                                id: `tab-${this.instance}-${i}`,
                                role: "tab",
                                tabindex: this.focused_tab == i ? 0 : -1,
                                "aria-selected": this.active_tab == i,
                                "aria-controls": `panel-${this.instance}-${i}`,
                                onclick: e => (this.active_tab = this.focused_tab = i),
                            },
                            m.trust(tab)
                        )
                    )
                ),
                vnode.attrs.tabs.panels.map((panel, i) =>
                    m(
                        ".tabs-body",
                        {
                            key: i,
                            id: `panel-${this.instance}-${i}`,
                            role: "tabpanel",
                            tabindex: 0,
                            hidden: this.active_tab != i,
                            "aria-labelledby": `tab-${this.instance}-${i}`,
                        },
                        m.trust(panel)
                    )
                ),
            ]
        );
    },
};

function Tabs(selector) {
    let tabs = { header: [], panels: [] };

    /**
     * @type {Element}
     */
    let target = document.querySelector(selector);

    // extract tab contents
    for (let child of target.children) {
        const tab = child.querySelector("[role=tab]");
        const panel = child.querySelector("[role=tabpanel]");

        tabs.header.push(tab ? tab.innerHTML : "");
        tabs.panels.push(panel ? panel.innerHTML : "");
    }

    m.mount(target, { view: () => m(MTabs, { tabs: tabs }) });
}
