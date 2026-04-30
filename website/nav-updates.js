const STORAGE_PREFIX = "harper.site.seen.";

function isAtPageBottom() {
    const scrollBottom = window.scrollY + window.innerHeight;
    const threshold = 16;
    return scrollBottom >= document.documentElement.scrollHeight - threshold;
}

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

    const renderIndicators = () => {
        document.querySelectorAll(".nav a[data-update-target]").forEach((link) => {
            const targetHref = link.getAttribute("href");
            const latestVersion = targetHref ? updates[targetHref] : null;
            if (!targetHref || !latestVersion) {
                return;
            }

            const seenVersion = localStorage.getItem(`${STORAGE_PREFIX}${targetHref}`);
            const hasUnread = seenVersion !== latestVersion;
            link.classList.toggle("has-update", hasUnread);

            if (!link.dataset.updateBound) {
                link.addEventListener("click", () => {
                    localStorage.setItem(`${STORAGE_PREFIX}${targetHref}`, latestVersion);
                });
                link.dataset.updateBound = "true";
            }
        });
    };

    renderIndicators();

    if (!currentVersion) {
        return;
    }

    const storageKey = `${STORAGE_PREFIX}${currentPath}`;
    const markCurrentPageSeen = () => {
        localStorage.setItem(storageKey, currentVersion);
        renderIndicators();
        window.removeEventListener("scroll", handleScroll);
    };

    const handleScroll = () => {
        if (isAtPageBottom()) {
            markCurrentPageSeen();
        }
    };

    if (localStorage.getItem(storageKey) !== currentVersion) {
        if (isAtPageBottom()) {
            markCurrentPageSeen();
        } else {
            window.addEventListener("scroll", handleScroll, { passive: true });
        }
    }
}

applyNavUpdateIndicators().catch(() => {});
