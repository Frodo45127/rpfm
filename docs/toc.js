// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><a href="chapter_1.html"><strong aria-hidden="true">1.</strong> What&#39;s RPFM?</a></li><li class="chapter-item expanded "><a href="chapter_2.html"><strong aria-hidden="true">2.</strong> Initial Configuration</a></li><li class="chapter-item expanded "><a href="chapter_3_0.html"><strong aria-hidden="true">3.</strong> Buttons and What They Do</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="chapter_3_1_0.html"><strong aria-hidden="true">3.1.</strong> Menu Bar</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="chapter_3_1_1.html"><strong aria-hidden="true">3.1.1.</strong> PackFile Menu</a></li><li class="chapter-item expanded "><a href="chapter_3_1_2.html"><strong aria-hidden="true">3.1.2.</strong> MyMod Menu</a></li><li class="chapter-item expanded "><a href="chapter_3_1_3.html"><strong aria-hidden="true">3.1.3.</strong> View Menu</a></li><li class="chapter-item expanded "><a href="chapter_3_1_4.html"><strong aria-hidden="true">3.1.4.</strong> Game Selected Menu</a></li><li class="chapter-item expanded "><a href="chapter_3_1_5.html"><strong aria-hidden="true">3.1.5.</strong> Special Stuff Menu</a></li><li class="chapter-item expanded "><a href="chapter_3_1_8.html"><strong aria-hidden="true">3.1.6.</strong> Tools Menu</a></li><li class="chapter-item expanded "><a href="chapter_3_1_6.html"><strong aria-hidden="true">3.1.7.</strong> About Menu</a></li><li class="chapter-item expanded "><a href="chapter_3_1_7.html"><strong aria-hidden="true">3.1.8.</strong> Debug Menu</a></li></ol></li><li class="chapter-item expanded "><a href="chapter_3_2_0.html"><strong aria-hidden="true">3.2.</strong> PackFile TreeView</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="chapter_3_2_1.html"><strong aria-hidden="true">3.2.1.</strong> Dependency Manager</a></li><li class="chapter-item expanded "><a href="chapter_3_2_2.html"><strong aria-hidden="true">3.2.2.</strong> PackFile Settings</a></li><li class="chapter-item expanded "><a href="chapter_3_2_3.html"><strong aria-hidden="true">3.2.3.</strong> Notes</a></li></ol></li><li class="chapter-item expanded "><a href="chapter_3_3_0.html"><strong aria-hidden="true">3.3.</strong> Global Search</a></li><li class="chapter-item expanded "><a href="chapter_3_4_0.html"><strong aria-hidden="true">3.4.</strong> Diagnostics Panel</a></li><li class="chapter-item expanded "><a href="chapter_3_5_0.html"><strong aria-hidden="true">3.5.</strong> Dependencies Panel</a></li><li class="chapter-item expanded "><a href="chapter_3_6_0.html"><strong aria-hidden="true">3.6.</strong> References Panel</a></li><li class="chapter-item expanded "><a href="chapter_3_7.html"><strong aria-hidden="true">3.7.</strong> Quick Notes Panel</a></li></ol></li><li class="chapter-item expanded "><a href="chapter_4_0.html"><strong aria-hidden="true">4.</strong> Editable Files</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="chapter_4_1.html"><strong aria-hidden="true">4.1.</strong> DB Tables</a></li><li class="chapter-item expanded "><a href="chapter_4_2.html"><strong aria-hidden="true">4.2.</strong> Locs</a></li><li class="chapter-item expanded "><a href="chapter_4_3.html"><strong aria-hidden="true">4.3.</strong> Text</a></li><li class="chapter-item expanded "><a href="chapter_4_4.html"><strong aria-hidden="true">4.4.</strong> Images</a></li><li class="chapter-item expanded "><a href="chapter_4_5.html"><strong aria-hidden="true">4.5.</strong> CA_VP8</a></li><li class="chapter-item expanded "><a href="chapter_4_6.html"><strong aria-hidden="true">4.6.</strong> AnimPacks</a></li><li class="chapter-item expanded "><a href="chapter_4_7.html"><strong aria-hidden="true">4.7.</strong> AnimTables</a></li><li class="chapter-item expanded "><a href="chapter_4_8.html"><strong aria-hidden="true">4.8.</strong> AnimFragments</a></li><li class="chapter-item expanded "><a href="chapter_4_9.html"><strong aria-hidden="true">4.9.</strong> MatchedCombat Tables</a></li><li class="chapter-item expanded "><a href="chapter_4_10.html"><strong aria-hidden="true">4.10.</strong> Portrait Settings</a></li><li class="chapter-item expanded "><a href="chapter_4_11.html"><strong aria-hidden="true">4.11.</strong> Audio</a></li><li class="chapter-item expanded "><a href="chapter_4_12.html"><strong aria-hidden="true">4.12.</strong> RigidModels</a></li></ol></li><li class="chapter-item expanded "><a href="chapter_5.html"><strong aria-hidden="true">5.</strong> DB Decoder</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="chapter_5_1.html"><strong aria-hidden="true">5.1.</strong> DB Types</a></li></ol></li><li class="chapter-item expanded "><a href="chapter_appendix.html"><strong aria-hidden="true">6.</strong> Extras</a></li><li class="chapter-item expanded "><a href="chapter_tutorials_intro.html"><strong aria-hidden="true">7.</strong> Tutorials</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="chapter_tutorials_translator.html"><strong aria-hidden="true">7.1.</strong> How To Translate A Mod</a></li><li class="chapter-item expanded "><a href="chapter_tutorials_optimizer.html"><strong aria-hidden="true">7.2.</strong> How To Optimize Your Mod</a></li><li class="chapter-item expanded "><a href="chapter_tutorials_twad_key_deletes.html"><strong aria-hidden="true">7.3.</strong> Datacores And You</a></li></ol></li><li class="chapter-item expanded "><a href="chapter_comp.html"><strong aria-hidden="true">8.</strong> Compilation Instructions</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
