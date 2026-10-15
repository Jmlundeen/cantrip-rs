// Unregisters any service worker an earlier build of this client registered, and empties its caches. That worker
// served whatever build it had cached, so a browser that once ran it kept booting an old client after the server had
// a new one. This build registers none (see Cantrip.Client.Web.csproj); this clears up after the ones that did.
if ("serviceWorker" in navigator) {
    navigator.serviceWorker.getRegistrations().then(async registrations => {
        if (registrations.length === 0) {
            return;
        }

        await Promise.all(registrations.map(registration => registration.unregister()));

        if ("caches" in globalThis) {
            const keys = await caches.keys();
            await Promise.all(keys.map(key => caches.delete(key)));
        }

        // The page itself may have come from the old worker, so load it once more from the server.
        location.reload();
    });
}
