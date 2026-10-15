// The browser tab's title, set from what the page is showing (PageTitle.cs, BrowserTitle.cs).
globalThis.cantripSetTitle = function (title) {
    if (document.title !== title) {
        document.title = title;
    }
};
