// A seat ticket is a bearer credential for one seat, so it is taken out of the address bar as the page loads, before
// it can be copied, bookmarked or kept in history, and held here for ClientConfig to read once. A reload resumes the
// seat from the membership the ticket bought, which the tab keeps in sessionStorage (SeatMemory.cs).
//
// It comes after the hash (#ticket=...) from the Discord bot's links, since a fragment is never sent to any server and
// so is in no request log; or in the query (?ticket=...) from anything that hands one over by hand.
(function () {
    const url = new URL(window.location.href);
    const fragment = new URLSearchParams(url.hash.slice(1));
    const ticket = fragment.get("ticket") || url.searchParams.get("ticket");
    if (ticket) {
        globalThis.cantripTicket = ticket;
        url.searchParams.delete("ticket");
        if (fragment.has("ticket")) {
            url.hash = "";
        }
        window.history.replaceState(window.history.state, "", url.toString());
    }
})();

// A seat moved to another table - two forming tables short of a player made into one - goes there: the page loads
// again with the new table's address in its query and the new ticket after the hash, where the code above takes it as
// any ticket is taken. Where card art is, the new table says itself (TableMessage.Art).
globalThis.cantripGoToTable = function (server, ticket) {
    const url = new URL(window.location.href);
    url.searchParams.set("server", server);
    url.hash = "ticket=" + encodeURIComponent(ticket);
    window.location.replace(url.toString());
};
