/** Base Screen implementation */
export default class Screen extends HTMLElement {
    constructor(templateId) {
        super();
        this.templateId = templateId;
    }

    connectedCallback() {
        this.classList.add("screen");
        if (this.templateId) {
            const template = document.getElementById(this.templateId);
            if (template) this.appendChild(template.content.cloneNode(true));
        }

        if (typeof this.onReady === "function") {
            this.onReady();
        }
    }

    onNavigate(_args) {}
}
