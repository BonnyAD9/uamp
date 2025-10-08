export class SearchBar extends HTMLElement {
    constructor() {
        super();
        const template = document.getElementById("search-input");
        this.innerHTML = "";
        this.appendChild(template.content.cloneNode(true));
    }

    connectedCallback() {
        const template = document.getElementById("search-input");
        this.innerHTML = "";
        this.appendChild(template.content.cloneNode(true));

        const input = this.querySelector("input");
        const clear = this.querySelector("button");

        if (this.hasAttribute("placeholder"))
            input.placeholder = this.getAttribute("placeholder");
        if (this.hasAttribute("id")) input.id = this.getAttribute("id");

        input.addEventListener("input", () => {
            this.dispatchEvent(
                new CustomEvent("input", {
                    detail: input.value,
                    bubbles: true,
                }),
            );
        });

        clear.addEventListener("click", () => {
            input.value = "";
            input.dispatchEvent(new Event("input", { bubbles: true }));
        });
    }
}

if (!customElements.get("search-bar"))
    customElements.define("search-bar", SearchBar);
