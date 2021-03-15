var diff = new diff({ Timeout: 0 });
var current_diff_list = [];

const TOKEN_PUNCT = -1;
const TOKEN_SPACE = 0;
const TOKEN_WORD = 1;
const DIFF_REPLACE = 2;

function htmlEntities(str) {
    return String(str).replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

function filter_user_text(id) {
    const domNode = document.getElementById(id);
    let changeset = Array.from(domNode.getElementsByClassName("change"));

    for (let change of changeset) {
        if (change.childNodes[0].nodeName == "DEL") {
            change.replaceWith(change.childNodes[0].innerText);
        } else {
            change.remove();
        }
    }

    return domNode.innerText;
}

function icon(name) {
    const klass = "icon";
    const variant = "outline";
    const svgpath = `/static/icons/teenyicons/${variant}.svg#${variant}`;

    return `<svg class="${klass}"><use xlink:href="${svgpath}--${name}" /></svg>`;
}

/**
 * Regroup diffs of replacements like spellcheck fixes into individual word replacement groups,
 * as the current diff algorithm doesn't do it on a word by word basis
 *
 * e.g. the default behaviour:
 * diff("abc def ghj", "abc ijk lmn") == [[0, "abc "], [-1, "def ghj"], [1, "ijk lmn"]]
 *
 * the regrouped output:
 * diff("abc def ghj", "abc ijk lmn") == [[0, "abc "], [2, "def", "ijk"], [0, " "], [2, "ghj", "lmn"]]
 */
function regroup_replacements(difflist) {
    let result = [];

    for (let i = 0; i < difflist.length; i++) {
        const curr = difflist[i];
        const next = difflist[i + 1];

        // replacements
        if (curr[0] == DIFF_DELETE && next && next[0] == DIFF_INSERT) {
            // generate new more fine grained diff
            let pdiff = diff.main(curr[1], next[1]);

            for (let j = 0; j < pdiff.length; j++) {
                const pcurr = pdiff[j];
                const pnext = pdiff[j + 1];

                if (pnext && pcurr[0] == DIFF_DELETE && pnext[0] == DIFF_INSERT) {
                    result.push([DIFF_REPLACE, pcurr[1], pnext[1]]);
                    j++;
                } else {
                    result.push(pcurr);
                }
            }

            i++;
        } else {
            result.push(curr);
        }
    }

    return result;
}

function diff_into_html(difflist) {
    let result = [];

    for (let i = 0; i < difflist.length; i++) {
        let diff = difflist[i];
        const change = htmlEntities(diff[1]).replaceAll("\n", "<br>");
        const replace = diff[2] ? htmlEntities(diff[2]).replaceAll("\n", "<br>") : null;

        switch (diff[0]) {
            case DIFF_INSERT:
                result.push(`<span
                        id="diff-${i}"
                        onclick="scroll_card_into_view(${i})"
                        onmouseover="activate_card(${i})"
                        onmouseleave="deactivate_card(${i})"
                        class="change"
                    ><ins>${change}</ins></span>`);
                break;
            case DIFF_DELETE:
                result.push(`<span
                        id="diff-${i}"
                        onclick="scroll_card_into_view(${i})"
                        onmouseover="activate_card(${i})"
                        onmouseleave="deactivate_card(${i})"
                        class="change"
                    ><del>${change}</del></span>`);
                break;
            case DIFF_REPLACE:
                result.push(`<span
                        id="diff-${i}"
                        onclick="scroll_card_into_view(${i})"
                        onmouseover="activate_card(${i})"
                        onmouseleave="deactivate_card(${i})"
                        class="change rep"
                    ><del>${change}</del><ins>${replace}</ins></span>`);
                break;
            case DIFF_EQUAL:
                result.push(`<span id="diff-${i}">${change}</span>`);
                break;
        }
    }

    return result.join("");
}

function scroll_diff_into_view(id) {
    document.getElementById(`diff-${id}`).scrollIntoView({ block: "center", inline: "center", behavior: "smooth" });
}

function scroll_card_into_view(id) {
    document
        .getElementById(`action-card-${id}`)
        .scrollIntoView({ block: "center", inline: "center", behavior: "smooth" });
}

function activate_card(id) {
    document.getElementById(`action-card-${id}`).classList.add("active");
}

function deactivate_card(id) {
    document.getElementById(`action-card-${id}`).classList.remove("active");
}

function accept_correction(e) {
    console.log("accept", this);
    let change = document.getElementById(this.dataset.linkTo);

    this.closest(".card").remove();

    if (change.childNodes[0].nodeName == "DEL" && !change.childNodes[1]) {
        // deletion
        change.remove();
    } else if (change.childNodes[1]) {
        // replacement insertion
        change.replaceWith(change.childNodes[1].innerText);
    } else {
        // insertion
        change.replaceWith(change.childNodes[0].innerText);
    }
}

function report_correction(e) {
    console.log("report", this);
    let id = this.dataset.linkTo.split("-")[1];

    const data = `diff=${encodeURIComponent(JSON.stringify(current_diff_list))}&ident=${encodeURIComponent(id)}`;

    const xhr = new XMLHttpRequest();
    xhr.open("POST", "/report");
    xhr.setRequestHeader("content-type", "application/x-www-form-urlencoded");
    xhr.send(data);

    let change = document.getElementById(this.dataset.linkTo);
    this.closest(".card").remove();

    if (change.childNodes[0].nodeName == "DEL") {
        change.replaceWith(change.childNodes[0].innerText);
    } else {
        change.remove();
    }
}

function ignore_correction(e) {
    console.log("ignore", this);
    let change = document.getElementById(this.dataset.linkTo);

    this.closest(".card").remove();

    if (change.childNodes[0].nodeName == "DEL") {
        change.replaceWith(change.childNodes[0].innerText);
    } else {
        change.remove();
    }
}

function accept_all_corrections() {
    Array.from(document.getElementsByClassName("change")).map((change) => {
        if (change.childNodes[0].nodeName == "DEL" && !change.childNodes[1]) {
            // deletion
            change.remove();
        } else if (change.childNodes[1]) {
            // replacement insertion
            change.replaceWith(change.childNodes[1].innerText);
        } else {
            // insertion
            change.replaceWith(change.childNodes[0].innerText);
        }
    });
    document.getElementById("correction-list").innerHTML = "";
}

function ignore_all_corrections() {
    Array.from(document.getElementsByClassName("change")).map((change) => {
        if (change.childNodes[0].nodeName == "DEL") {
            change.replaceWith(change.childNodes[0].innerText);
        } else {
            change.remove();
        }
    });
    document.getElementById("correction-list").innerHTML = "";
}

function fetch_corrections(callback) {
    document.getElementById("corrections-submit").classList.add("loading");

    let userText = filter_user_text("data");

    let request = new XMLHttpRequest();
    request.open("POST", "/editor");
    request.send(userText);

    request.onreadystatechange = function () {
        if (this.readyState == 4 && this.status == 200) {
            let diffResult = diff.main(userText, this.response);
            diff.cleanupSemantic(diffResult);

            diffResult = regroup_replacements(diffResult);
            callback(diffResult);
            document.getElementById("corrections-submit").classList.remove("loading");
        } else if (this.readyState == 4) {
            console.log("failure");
        }
    };
}

function show_corrections_editor(corrections) {
    document.getElementById("data").innerHTML = diff_into_html(corrections);
}

function card_mouse_hover(e) {
    document.getElementById(this.dataset.linkTo).classList.add("active");
}

function card_mouse_leave(e) {
    document.getElementById(this.dataset.linkTo).classList.remove("active");
}

function show_corrections_panel(corrections) {
    const correctionList = document.getElementById("correction-list");
    const newCorrections = [];

    for (let i = 0; i < corrections.length; i++) {
        let diff = corrections[i];

        if (diff[0] == DIFF_EQUAL) {
            continue;
        }

        const prune_context = (j) => corrections[j][1].split(/\s/).filter((e) => e != "");
        const get_context_left = (offset) => (corrections[i - offset] && prune_context(i - offset).pop()) || "";
        const get_context_right = (offset) => (corrections[i + offset] && prune_context(i + offset).shift()) || "";

        let ctxLeft = get_context_left(1);
        let ctxRight = get_context_right(1);

        ctxLeft = ctxLeft || (corrections[i - 1] ? corrections[i - 1][1] : "") + get_context_left(2);
        ctxRight = ctxRight || (corrections[i + 1] ? corrections[i + 1][1] : "") + get_context_right(2);

        let change;

        if (diff[0] == DIFF_REPLACE) {
            change = `<div class="context">
                ${ctxLeft} <span class="rep">${diff[1]} ${icon("arrow-right")} ${diff[2]}</span> ${ctxRight}
            </div>`;
        } else {
            let changeType = diff[0] == DIFF_INSERT ? "ins" : "del";
            change = `<div class="context">${ctxLeft} <${changeType}>${diff[1]}</${changeType}> ${ctxRight}</div>`;
        }

        let card = document.createElement("div");
        card.classList.add("card");
        card.id = `action-card-${i}`;
        card.dataset.linkTo = `diff-${i}`;
        card.addEventListener("mouseover", card_mouse_hover);
        card.addEventListener("mouseover", function () {
            scroll_diff_into_view(i);
        });
        card.addEventListener("mouseleave", card_mouse_leave);
        card.innerHTML = change;

        let actions = document.createElement("div");
        actions.classList.add("actions");

        let actionAccept = document.createElement("button");
        actionAccept.innerHTML = icon("tick-circle");
        actionAccept.classList.add("accept");
        actionAccept.title = "Accepter";
        actionAccept.dataset.linkTo = `diff-${i}`;
        actionAccept.addEventListener("click", accept_correction);

        let actionReport = document.createElement("button");
        actionReport.innerHTML = icon("flag");
        actionReport.classList.add("report");
        actionReport.title = "Rapporter";
        actionReport.dataset.linkTo = `diff-${i}`;
        actionReport.addEventListener("click", report_correction);

        let actionIgnore = document.createElement("button");
        actionIgnore.innerHTML = icon("bin");
        actionIgnore.classList.add("ignore");
        actionIgnore.title = "Ignorer";
        actionIgnore.dataset.linkTo = `diff-${i}`;
        actionIgnore.addEventListener("click", ignore_correction);

        actions.append(actionAccept, actionIgnore, actionReport);
        card.append(actions);
        newCorrections.push(card);
    }

    correctionList.innerHTML = "";
    newCorrections.map((n) => correctionList.appendChild(n));
}

function get_and_show_corrections() {
    fetch_corrections((corrections) => {
        current_diff_list = corrections;
        show_corrections_editor(corrections);
        show_corrections_panel(corrections);
    });
}

function editor_paste_as_plaintext(e) {
    e.preventDefault();

    const text = (e.originalEvent || e).clipboardData.getData("text/plain");

    if (document.queryCommandSupported("insertText")) {
        document.execCommand("insertText", false, text);
    } else {
        document.execCommand("paste", false, text);
    }
}

document.addEventListener("readystatechange", () => {
    const editor = document.getElementById("data");

    editor.addEventListener("focusout", function (e) {
        if (!this.textContent.trim().length) {
            this.textContent = "";
        }
    });

    editor.addEventListener("paste", editor_paste_as_plaintext);
});

function qmodal(id) {
    let el = document.getElementById(id);
    el.classList.toggle("active");

    let firstInput = el.getElementsByTagName("input")[0];
    if (firstInput) {
        firstInput.focus();
    }
}

function submit_feedback(e) {
    e.preventDefault();

    const form = document.forms["feedback-form"];
    const data = [
        `comma_quality=${encodeURIComponent(form.comma_quality.value)}`,
        `wait_time=${encodeURIComponent(form.wait_time.value)}`,
        `problems=${encodeURIComponent(form.problems.value)}`,
        `thoughts=${encodeURIComponent(form.thoughts.value)}`,
    ].join("&");

    const xhr = new XMLHttpRequest();
    xhr.open("POST", "/feedback");
    xhr.setRequestHeader("content-type", "application/x-www-form-urlencoded");
    xhr.send(data);

    qmodal("modal-feedback");
    qmodal("modal-result");
}
