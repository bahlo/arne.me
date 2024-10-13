;; Don't flicker GUI elements on startup
(push '(menu-bar-lines . 0) default-frame-alist)
(push '(tool-bar-lines . 0) default-frame-alist)
(push '(vertical-scroll-bars) default-frame-alist)

;; We're using straight.el instead of package.el, no need to load it
(setq package-enable-at-startup nil)
