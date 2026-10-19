// A page somewhere else, opened in a new tab so the table stays open (ExternalLink.cs): only an https address.
globalThis.cantripOpenTab = function (address) {
    const url = new URL(address);
    if (url.protocol === "https:") {
        window.open(url.toString(), "_blank", "noopener");
    }
};
