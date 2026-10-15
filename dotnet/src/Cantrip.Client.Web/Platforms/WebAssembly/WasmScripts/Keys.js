// Every key pressed on the page, queued for the client to read each frame (see KeyboardBridge.cs). Listened for on the
// window itself, because Uno delivers keys only to a focused control, and clicking the table - a canvas - focuses none:
// the Pass key did nothing until something that takes focus had been clicked. A key typed into a text field, such as a
// card being named, is the field's, and is marked so the client leaves it alone.
(function () {
    const queue = [];

    const typing = (e) => {
        const target = e.target;
        return !!target && (target.tagName === "INPUT" || target.tagName === "TEXTAREA" || target.isContentEditable);
    };

    const note = (kind, e) => queue.push([
        kind,
        e.keyCode,
        e.repeat ? 1 : 0,
        e.ctrlKey ? 1 : 0,
        e.shiftKey ? 1 : 0,
        e.altKey ? 1 : 0,
        typing(e) ? 1 : 0,
    ].join(","));

    window.addEventListener("keydown", (e) => {
        note("d", e);

        // Space is the Pass key, and a page that scrolled under it would move the table.
        if (!typing(e) && e.keyCode === 32) {
            e.preventDefault();
        }
    }, true);

    window.addEventListener("keyup", (e) => note("u", e), true);

    // Keys let go of while the window is not looking are never heard going up.
    window.addEventListener("blur", () => queue.push("b"));

    globalThis.cantripTakeKeys = () => {
        const taken = queue.join("\n");
        queue.length = 0;
        return taken;
    };
})();
