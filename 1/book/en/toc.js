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
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded affix "><a href="preface.html">Preface</a></li><li class="chapter-item expanded "><a href="by-example.html"><strong aria-hidden="true">1.</strong> RTIC by example</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="by-example/app.html"><strong aria-hidden="true">1.1.</strong> The app</a></li><li class="chapter-item expanded "><a href="by-example/resources.html"><strong aria-hidden="true">1.2.</strong> Resources</a></li><li class="chapter-item expanded "><a href="by-example/app_init.html"><strong aria-hidden="true">1.3.</strong> The init task</a></li><li class="chapter-item expanded "><a href="by-example/app_idle.html"><strong aria-hidden="true">1.4.</strong> The idle task</a></li><li class="chapter-item expanded "><a href="by-example/app_task.html"><strong aria-hidden="true">1.5.</strong> Defining tasks</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="by-example/hardware_tasks.html"><strong aria-hidden="true">1.5.1.</strong> Hardware tasks</a></li><li class="chapter-item expanded "><a href="by-example/software_tasks.html"><strong aria-hidden="true">1.5.2.</strong> Software tasks &amp; spawn</a></li><li class="chapter-item expanded "><a href="by-example/message_passing.html"><strong aria-hidden="true">1.5.3.</strong> Message passing &amp; capacity</a></li><li class="chapter-item expanded "><a href="by-example/app_priorities.html"><strong aria-hidden="true">1.5.4.</strong> Task priorities</a></li><li class="chapter-item expanded "><a href="by-example/monotonic.html"><strong aria-hidden="true">1.5.5.</strong> Monotonic &amp; spawn_{at/after}</a></li></ol></li><li class="chapter-item expanded "><a href="by-example/starting_a_project.html"><strong aria-hidden="true">1.6.</strong> Starting a new project</a></li><li class="chapter-item expanded "><a href="by-example/app_minimal.html"><strong aria-hidden="true">1.7.</strong> The minimal app</a></li><li class="chapter-item expanded "><a href="by-example/tips.html"><strong aria-hidden="true">1.8.</strong> Tips &amp; Tricks</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="by-example/tips_monotonic_impl.html"><strong aria-hidden="true">1.8.1.</strong> Implementing Monotonic</a></li><li class="chapter-item expanded "><a href="by-example/tips_destructureing.html"><strong aria-hidden="true">1.8.2.</strong> Resource de-structure-ing</a></li><li class="chapter-item expanded "><a href="by-example/tips_indirection.html"><strong aria-hidden="true">1.8.3.</strong> Avoid copies when message passing</a></li><li class="chapter-item expanded "><a href="by-example/tips_static_lifetimes.html"><strong aria-hidden="true">1.8.4.</strong> &#39;static super-powers</a></li><li class="chapter-item expanded "><a href="by-example/tips_view_code.html"><strong aria-hidden="true">1.8.5.</strong> Inspecting generated code</a></li><li class="chapter-item expanded "><a href="by-example/tips_from_ram.html"><strong aria-hidden="true">1.8.6.</strong> Running tasks from RAM</a></li></ol></li></ol></li><li class="chapter-item expanded "><a href="awesome_rtic.html"><strong aria-hidden="true">2.</strong> Awesome RTIC examples</a></li><li class="chapter-item expanded "><a href="migration.html"><strong aria-hidden="true">3.</strong> Migration Guides</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="migration/migration_v5.html"><strong aria-hidden="true">3.1.</strong> v0.5.x to v1.0.x</a></li><li class="chapter-item expanded "><a href="migration/migration_v4.html"><strong aria-hidden="true">3.2.</strong> v0.4.x to v0.5.x</a></li><li class="chapter-item expanded "><a href="migration/migration_rtic.html"><strong aria-hidden="true">3.3.</strong> RTFM to RTIC</a></li></ol></li><li class="chapter-item expanded "><a href="internals.html"><strong aria-hidden="true">4.</strong> Under the hood</a></li><li><ol class="section"><li class="chapter-item expanded "><a href="internals/targets.html"><strong aria-hidden="true">4.1.</strong> Cortex-M architectures</a></li></ol></li></ol>';
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
