// Entry point of the Stammtisch web app.

import './style.css';
import { App } from './app';
import { installSprite } from './ui/art';

const root = document.getElementById('app');
installSprite();
if (root) {
  void new App(root).start();
}
