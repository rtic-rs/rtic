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
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="preface.html">Preface</a></li><li class="chapter-item expanded affix "><li class="spacer"></li><li class="chapter-item expanded "><a href="starting_a_project.html"><strong aria-hidden="true">1.</strong> Starting a new project</a></li><li class="chapter-item expanded "><a href="by-example.html"><strong aria-hidden="true">2.</strong> RTIC by example</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="by-example/app.html"><strong aria-hidden="true">2.1.</strong> The app</a></li><li class="chapter-item expanded "><a href="by-example/hardware_tasks.html"><strong aria-hidden="true">2.2.</strong> Hardware tasks</a></li><li class="chapter-item expanded "><a href="by-example/software_tasks.html"><strong aria-hidden="true">2.3.</strong> Software tasks &amp; spawn</a></li><li class="chapter-item expanded "><a href="by-example/resources.html"><strong aria-hidden="true">2.4.</strong> Resources</a></li><li class="chapter-item expanded "><a href="by-example/app_init.html"><strong aria-hidden="true">2.5.</strong> The init task</a></li><li class="chapter-item expanded "><a href="by-example/app_idle.html"><strong aria-hidden="true">2.6.</strong> The idle task</a></li><li class="chapter-item expanded "><a href="by-example/channel.html"><strong aria-hidden="true">2.7.</strong> Channel based communication</a></li><li class="chapter-item expanded "><a href="by-example/delay.html"><strong aria-hidden="true">2.8.</strong> Delay and Timeout using Monotonics</a></li><li class="chapter-item expanded "><a href="by-example/app_minimal.html"><strong aria-hidden="true">2.9.</strong> The minimal app</a></li><li class="chapter-item expanded "><a href="by-example/tips/index.html"><strong aria-hidden="true">2.10.</strong> Tips &amp; Tricks</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="by-example/tips/destructureing.html"><strong aria-hidden="true">2.10.1.</strong> Resource de-structure-ing</a></li><li class="chapter-item expanded "><a href="by-example/tips/indirection.html"><strong aria-hidden="true">2.10.2.</strong> Avoid copies when message passing</a></li><li class="chapter-item expanded "><a href="by-example/tips/static_lifetimes.html"><strong aria-hidden="true">2.10.3.</strong> &#39;static super-powers</a></li><li class="chapter-item expanded "><a href="by-example/tips/view_code.html"><strong aria-hidden="true">2.10.4.</strong> Inspecting generated code</a></li></ol></li></ol></li><li class="chapter-item expanded "><a href="monotonic_impl.html"><strong aria-hidden="true">3.</strong> Monotonics &amp; the Timer Queue</a></li><li class="chapter-item expanded "><a href="rtic_vs.html"><strong aria-hidden="true">4.</strong> RTIC vs. the world</a></li><li class="chapter-item expanded "><a href="rtic_and_embassy.html"><strong aria-hidden="true">5.</strong> RTIC and Embassy</a></li><li class="chapter-item expanded "><a href="awesome_rtic.html"><strong aria-hidden="true">6.</strong> Awesome RTIC examples</a></li><li class="chapter-item expanded affix "><li class="spacer"></li><li class="chapter-item expanded "><a href="migration_v1_v2.html"><strong aria-hidden="true">7.</strong> Migrating from v1.0.x to v2.0.0</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="migration_v1_v2/monotonics.html"><strong aria-hidden="true">7.1.</strong> Migrating to rtic-monotonics</a></li><li class="chapter-item expanded "><a href="migration_v1_v2/async_tasks.html"><strong aria-hidden="true">7.2.</strong> Software tasks must now be async</a></li><li class="chapter-item expanded "><a href="migration_v1_v2/rtic-sync.html"><strong aria-hidden="true">7.3.</strong> Using and understanding rtic-sync</a></li><li class="chapter-item expanded "><a href="migration_v1_v2/complete_example.html"><strong aria-hidden="true">7.4.</strong> A code example on migration</a></li></ol></li><li class="chapter-item expanded "><li class="spacer"></li><li class="chapter-item expanded "><a href="internals.html"><strong aria-hidden="true">8.</strong> Under the hood</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="internals/targets.html"><strong aria-hidden="true">8.1.</strong> Target architectures</a></li></ol></li></ol>';
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
