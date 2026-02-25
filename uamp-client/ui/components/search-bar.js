export default class SearchBar extends HTMLElement {
    constructor() {
        super();

        this.timeout = 100;
        this.searchTimeout = null;
    }

    connectedCallback() {
        const template = document.getElementById("search-input");
        this.appendChild(template.content.cloneNode(true));

        this.dom = {
            input: this.querySelector("input"),
            clear: this.querySelector("button"),
        };

        if (this.hasAttribute("placeholder"))
            this.dom.input.placeholder = this.getAttribute("placeholder");
        if (this.hasAttribute("id"))
            this.dom.input.id = this.getAttribute("id");

        this.timeout = Number(this.getAttribute("timeout")) || this.timeout;

        this.#setupListeners();
    }

    #setupListeners() {
        this.dom.input.addEventListener("input", (e) => {
            e.stopPropagation();
            clearTimeout(this.searchTimeout);
            this.searchTimeout = setTimeout(
                () => this.#dispatchEvent(),
                this.timeout,
            );
        });

        this.dom.clear.addEventListener("click", () => {
            this.dom.input.value = "";
            this.dom.input.dispatchEvent(new Event("input", { bubbles: true }));
        });
    }

    #dispatchEvent() {
        this.dispatchEvent(
            new CustomEvent("input", {
                detail: this.dom.input.value,
                bubbles: true,
            }),
        );
    }
}
