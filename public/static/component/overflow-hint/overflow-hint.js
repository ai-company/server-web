window.addEventListener("load", () => {
    const targetNode = document.querySelector("body");

    const o_intersection = new IntersectionObserver(
        intersections => intersections.map(e => e.target.classList.toggle("pinned", e.intersectionRatio < 1)),
        { threshold: 1 }
    );

    /**
     * @param {MutationRecord[]} mutations
     */
    const callback = function (mutations, observer) {
        for (let mutation of mutations) {
            for (let node of mutation.addedNodes) {
                if (
                    node.classList &&
                    (node.classList.contains("overflow-hint-top") || node.classList.contains("overflow-hint-bottom"))
                ) {
                    o_intersection.observe(node);
                }

                if (node.querySelector) {
                    let children = node.querySelectorAll('[class*="overflow-hint"]');
                    if (children) {
                        for (let child of children) {
                            o_intersection.observe(child);
                        }
                    }
                }
            }
        }
    };

    const o_mutation = new MutationObserver(callback);
    o_mutation.observe(targetNode, { childList: true, subtree: true });
});
