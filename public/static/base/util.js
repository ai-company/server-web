function togglepassword(self, id) {
    target = document.getElementById(id);
    console.log(target);

    if (target.type == "password") {
        self.attributes["aria-label"].value = "Hide password";
        self.innerHTML =
            "<svg class=icon><use href='/static/icons/teenyicons/outline.svg#outline--eye-closed'></use></svg>";
        target.type = "text";
    } else {
        self.attributes["aria-label"].value = "Show password";
        self.innerHTML = "<svg class=icon><use href='/static/icons/teenyicons/outline.svg#outline--eye'></use></svg>";
        target.type = "password";
    }
}

function cvrapi(e, errorid) {
    let error = document.getElementById(errorid);

    if (!this.checkValidity()) {
        error.innerHTML = "The VAT number is invalid";
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
                        error.innerHTML = "The VAT number is invalid";
                        error.parentElement.classList.remove("d-none");
                        break;
                    case "NOT_FOUND":
                        error.innerHTML = "No company found associated with this VAT";
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
                        error.innerHTML = "The VAT number is invalid";
                        error.parentElement.classList.remove("d-none");
                        break;
                    case "NOT_FOUND":
                        error.innerHTML = "No company found associated with this VAT";
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
