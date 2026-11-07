import React from 'react';
import ReactDOM from 'react-dom/client';

import { App } from './App';
import './styles.css';

const container = document.getElementById('root');
if (!container) {
  throw new Error('Не найден корневой элемент #root');
}

// Стандартное меню браузера по правой кнопке мыши в редакторе не нужно:
// все действия доступны через меню приложения и горячие клавиши.
document.addEventListener('contextmenu', (event) => event.preventDefault());

ReactDOM.createRoot(container).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
);
