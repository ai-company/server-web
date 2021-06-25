// Copyright 2021, ai-company
// https://github.com/ai-company

/**
 * Modal component
 *
 * USAGE:
 * ```
 * <button class="submit" onclick="modal('my-modal')">Open Modal</button>
 *
 * <div id="my-modal" class="modal">
 *     <a class="modal-close" onclick="modal('my-modal')"></a>
 *
 *     <div class="modal-content panel">
 *         <a class="modal-x" onclick="modal('my-modal')">
 *             <svg class="icon">
 *                 <use href="/static/icons/teenyicons/outline.svg#outline--x"></use>
 *             </svg>
 *         </a>
 *
 *         <div>Modal text</div>
 *     </div>
 * </div>
 * ```
 */
function modal(id) {
    let el = document.getElementById(id);
    el.classList.toggle("active");

    let firstInput = el.getElementsByTagName("input")[0];
    if (firstInput) {
        firstInput.focus();
    }
}
