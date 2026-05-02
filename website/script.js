// Tiny landing-page interactions. No framework, no build step.

(() => {
  const REPO = "Frodo45127/rpfm";

  // Close any open <details> dropdown when clicking outside it, so the menu
  // doesn't linger after the user has decided not to pick anything.
  document.addEventListener("click", (e) => {
    document.querySelectorAll("details.cta-split-dropdown[open]").forEach((d) => {
      if (!d.contains(e.target)) d.removeAttribute("open");
    });
  });

  // "Latest pre-release" entries call into the GitHub API to find the most
  // recent release flagged as a pre-release, then redirect there. Falls back
  // to /releases (the full list) if the API isn't reachable.
  document.querySelectorAll('a[data-channel="prerelease"]').forEach((a) => {
    a.addEventListener("click", async (e) => {
      e.preventDefault();
      const fallback = `https://github.com/${REPO}/releases`;
      try {
        const resp = await fetch(`https://api.github.com/repos/${REPO}/releases?per_page=30`, {
          headers: { Accept: "application/vnd.github+json" },
        });
        if (!resp.ok) throw new Error(`HTTP ${resp.status}`);
        const releases = await resp.json();
        const pre = releases.find((r) => r.prerelease && !r.draft);
        window.location.href = pre ? pre.html_url : fallback;
      } catch (err) {
        console.warn("pre-release lookup failed:", err);
        window.location.href = fallback;
      }
    });
  });
})();
