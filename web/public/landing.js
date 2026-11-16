// The landing page's only script (web/start.html, web/en/start.html; DESIGN
// 16.9). A classic script in <head>, loaded synchronously, so it decides
// before the page is drawn. The CSP allows no inline script, hence a file.
// It holds no game rule and reads only whether this browser played before.
//
// - Returning players (the game left its marks in localStorage) go on to
//   the game at /spielen; a shared link (#seed-..., #umzug=..., #spielstand=...)
//   goes there for everyone. Only the plain addresses / and /en forward;
//   /start and /en/start always show the page.
// - Language: on / a browser that is not German, or a stored choice of
//   English, goes to /en; on /en a stored choice of German goes to /. The
//   language links remember the choice under the game's own key, so the
//   game then speaks the same language.
(function () {
  'use strict';
  var GAME = '/spielen';
  // stammtisch-visited: written by the game at every start (web/src/main.ts);
  // meta and run: the game's saves, for players from before that key.
  var PLAYED = ['stammtisch-visited', 'stammtisch.meta', 'stammtisch.run'];
  var LANG_KEY = 'stammtisch.lang';
  var SHARED = /^#(seed-|umzug=|spielstand=)/;

  function read(key) {
    try {
      return window.localStorage.getItem(key);
    } catch (e) {
      return null;
    }
  }

  function write(key, value) {
    try {
      window.localStorage.setItem(key, value);
    } catch (e) {
      // Blocked storage: the link still works, the choice is not kept.
    }
  }

  function browserLanguage() {
    var list = [];
    try {
      list = navigator.languages && navigator.languages.length ? navigator.languages : [navigator.language];
    } catch (e) {
      list = [];
    }
    var first = String(list[0] || 'de').toLowerCase();
    return first.indexOf('de') === 0 ? 'de' : 'en';
  }

  function go(to) {
    // Hide the page while the next one loads (landing.css), no flash.
    document.documentElement.className += ' leaving';
    window.location.replace(to);
  }

  var html = document.documentElement;
  var page = html.getAttribute('lang') === 'en' ? 'en' : 'de';
  var path = window.location.pathname.replace(/\/+$/, '') || '/';
  var plain = path === '/' || path === '/en';
  var hash = window.location.hash;
  var search = window.location.search;

  if (SHARED.test(hash)) return go(GAME + search + hash);
  if (plain) {
    for (var i = 0; i < PLAYED.length; i++) {
      if (read(PLAYED[i]) !== null) return go(GAME + search + hash);
    }
    var stored = read(LANG_KEY);
    if (page === 'de' && (stored === 'en' || (stored === null && browserLanguage() === 'en'))) return go('/en' + search + hash);
    if (page === 'en' && stored === 'de') return go('/' + search + hash);
  }

  // The preview film: muted, looping. With reduced motion it stays on its
  // poster until asked; a button stops and starts it (its labels come
  // from the page, this file has no text of its own).
  document.addEventListener('DOMContentLoaded', function () {
    var video = document.querySelector('video[data-testid="landing-video"]');
    var toggle = document.querySelector('[data-video-toggle]');
    if (!video || !toggle) return;
    var still = false;
    try {
      still = window.matchMedia('(prefers-reduced-motion: reduce)').matches;
    } catch (e) {
      still = false;
    }
    function label() {
      toggle.textContent = toggle.getAttribute(video.paused ? 'data-label-play' : 'data-label-pause');
    }
    if (still) {
      video.removeAttribute('autoplay');
      video.pause();
    }
    video.addEventListener('play', label);
    video.addEventListener('pause', label);
    toggle.addEventListener('click', function () {
      if (video.paused) {
        var started = video.play();
        if (started && started.catch) started.catch(function () {});
      } else {
        video.pause();
      }
    });
    toggle.hidden = false;
    label();
  });

  document.addEventListener('click', function (event) {
    var target = event.target;
    var link = target && target.closest ? target.closest('a[data-lang]') : null;
    if (link) write(LANG_KEY, link.getAttribute('data-lang'));
  });
})();
