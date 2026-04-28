const STORAGE_PREFIX = "harper.site.seen.";

async function applyNavUpdateIndicators() {
    const currentPath = window.location.pathname.split("/").pop() || "index.html";
    const currentVersion = document
        .querySelector('meta[name="harper-update-version"]')
        ?.getAttribute("content");
    const response = await fetch("nav-updates.json", {
        headers: { Accept: "application/json" }
    });

    if (!response.ok) {
        return;
    }

    const updates = await response.json();

    if (currentVersion) {
        localStorage.setItem(`${STORAGE_PREFIX}${currentPath}`, currentVersion);
    }

    document.querySelectorAll(".nav a[data-update-target]").forEach((link) => {
        const targetHref = link.getAttribute("href");
        const latestVersion = targetHref ? updates[targetHref] : null;
        if (!targetHref || !latestVersion) {
            return;
        }

        const seenVersion = localStorage.getItem(`${STORAGE_PREFIX}${targetHref}`);
        const hasUnread = seenVersion !== latestVersion;
        link.classList.toggle("has-update", hasUnread);

        link.addEventListener("click", () => {
            localStorage.setItem(`${STORAGE_PREFIX}${targetHref}`, latestVersion);
        });
    });
}

applyNavUpdateIndicators().catch(() => {});
