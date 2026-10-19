// The deck editor's changes not yet saved: while there are any, the browser asks before the page is left or reloaded
// (DeckEditor in MainPage.Decks.cs says when). The browser words the question itself; a page cannot.
(function () {
    let unsaved = false;

    globalThis.cantripDeckUnsaved = function (value) {
        unsaved = !!value;
    };

    window.addEventListener("beforeunload", (e) => {
        if (unsaved) {
            e.preventDefault();
            e.returnValue = "";
        }
    });

    // Signing in goes to Discord and back: the page is left on purpose, so it is not asked about.
    globalThis.cantripGoTo = function (address) {
        window.location.assign(address);
    };
})();
