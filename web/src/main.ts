// Entry point of the Stammtisch web app.

import './style.css';
import { App } from './app';

const root = document.getElementById('app');
if (root) {
  void new App(root).start();
}
