// Copyright 2021, ai-company
// https://github.com/ai-company

function modal(id) {
    let el = document.getElementById(id);
    el.classList.toggle("active");

    let firstInput = el.getElementsByTagName("input")[0];
    if (firstInput) {
        firstInput.focus();
    }
}
