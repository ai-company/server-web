function togglepassword(self, id) {
    target = document.getElementById(id);
    console.log(target);

    if (target.type == "password") {
        self.attributes["aria-label"].value = "Skjul password";
        self.innerHTML =
            "<svg class=icon><use href='/static/icons/teenyicons/outline.svg#outline--eye-closed'></use></svg>";
        target.type = "text";
    } else {
        self.attributes["aria-label"].value = "Vis password";
        self.innerHTML = "<svg class=icon><use href='/static/icons/teenyicons/outline.svg#outline--eye'></use></svg>";
        target.type = "password";
    }
}

function cvrapi(e, errorid) {
    let error = document.getElementById(errorid);

    if (!this.checkValidity()) {
        error.innerHTML = "Det indtastede CVR-nummer er ugyldigt.";
        error.parentElement.classList.remove("d-none");
        return;
    } else {
        error.innerHTML = "";
        error.parentElement.classList.add("d-none");
    }

    let xhr = new XMLHttpRequest();
    xhr.open("GET", "http://cvrapi.dk/api?search=" + this.value + "&country=" + "dk");

    xhr.onreadystatechange = function (e) {
        if (xhr.readyState === XMLHttpRequest.DONE) {
            let status = xhr.status;
            let response = JSON.parse(xhr.responseText);

            if (status === 0 || (status >= 200 && status < 400)) {
                error.parentElement.classList.add("d-none");
                switch (response.error) {
                    case "QUOTA_EXCEEDED":
                    case "BANNED":
                    case "INVALID_UA":
                        document.getElementById("cvr").removeEventListener("blur", cvrapi);
                        break;
                    case "INVALID_VAT":
                        error.innerHTML = "Det indtastede CVR-nummer er ugyldigt.";
                        error.parentElement.classList.remove("d-none");
                        break;
                    case "NOT_FOUND":
                        error.innerHTML = "Der blev ikke fundet nogen virksomhed med dette CVR-nummer.";
                        error.parentElement.classList.remove("d-none");
                        break;
                    case "INTERNAL_ERROR":
                        // error.innerHTML = "INTERNAL_ERROR";
                        break;
                }
            } else if (status == 404) {
                switch (response.error) {
                    case "QUOTA_EXCEEDED":
                    case "BANNED":
                    case "INVALID_UA":
                        this.removeEventListener("blur", cvrapi);
                        break;
                    case "INVALID_VAT":
                        error.innerHTML = "Det indtastede CVR-nummer er ugyldigt.";
                        error.parentElement.classList.remove("d-none");
                        break;
                    case "NOT_FOUND":
                        error.innerHTML = "Der blev ikke fundet nogen virksomhed med dette CVR-nummer.";
                        error.parentElement.classList.remove("d-none");
                        break;
                    case "INTERNAL_ERROR":
                        // error.innerHTML = "INTERNAL_ERROR";
                        break;
                }
            }
        }
    };

    xhr.send();
}

function smoothscroll(e) {
    if (new URL(e.target.href).pathname == window.location.pathname) {
        e.preventDefault();
        window.history.pushState(null, null, e.target.href);
        document.querySelector(window.location.hash).scrollIntoView({ behavior: "smooth" });
    }
}

if (window.location.hash) {
    document.addEventListener("DOMContentLoaded", e => document.querySelector(window.location.hash).scrollIntoView());
}
