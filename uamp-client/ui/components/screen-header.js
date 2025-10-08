class ScreenHeader extends HTMLElement {
    constructor() {
        super();
    }

    connectedCallback() {
        const template = document.getElementById("screen-header");
        const inner = this.innerHTML;
        this.innerHTML = "";

        this.appendChild(template.content.cloneNode(true));
        const title = this.getAttribute("title") ?? "";
        this.querySelector("h1").textContent = title;

        this.querySelector(".filler").innerHTML = inner;

        const oninput = this.getAttribute("oninput");
        if (oninput) {
            const search = document.createElement("search-bar");
            const key = title.trim().toLowerCase();
            search.setAttribute("id", `${key}-search`);
            search.setAttribute("placeholder", `Search ${key}...`);
            search.setAttribute("oninput", oninput);
            this.querySelector(".header-row").appendChild(search);
        }
    }
}

if (!customElements.get("screen-header"))
    customElements.define("screen-header", ScreenHeader);
