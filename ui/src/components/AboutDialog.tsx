/**
 * Окно «О программе».
 */

import { useStore } from '../store';

export function AboutDialog() {
  const open = useStore((state) => state.aboutOpen);
  const closeAbout = useStore((state) => state.closeAbout);

  if (!open) return null;

  return (
    <div className="modal-backdrop" onClick={closeAbout}>
      <div className="modal about" onClick={(event) => event.stopPropagation()}>
        <div className="about-head">
          <span className="brand-mark large">FL</span>
          <div>
            <h2>Fontlord</h2>
            <p className="hint">Редактор шрифтов с векторными кистями</p>
          </div>
        </div>

        <p className="hint">
          Символы рисуются обычной мышью, а специальные кисти превращают рисунок в
          настоящие векторные контуры TrueType. Проект сохраняется в файл
          <code>.fontlord</code>, готовый шрифт экспортируется в форматах
          <code> .ttf</code>, <code>.otf</code>, <code>.woff</code> и{' '}
          <code>.woff2</code>.
        </p>

        <table className="about-table">
          <tbody>
            <tr>
              <td>Ctrl+N / Ctrl+O / Ctrl+S</td>
              <td>новый проект, открыть, сохранить</td>
            </tr>
            <tr>
              <td>Ctrl+E</td>
              <td>экспортировать шрифт</td>
            </tr>
            <tr>
              <td>Ctrl+Z / Ctrl+Y</td>
              <td>отменить, повторить</td>
            </tr>
            <tr>
              <td>B / M / N / S / E</td>
              <td>перо, маркер, каллиграфия, мягкая кисть, ластик</td>
            </tr>
            <tr>
              <td>Ctrl+0 / Ctrl++ / Ctrl+−</td>
              <td>масштаб холста</td>
            </tr>
            <tr>
              <td>Del / Esc</td>
              <td>удалить глиф, закрыть глиф</td>
            </tr>
          </tbody>
        </table>

        <div className="modal-actions">
          <button type="button" className="primary" onClick={closeAbout}>
            Закрыть
          </button>
        </div>
      </div>
    </div>
  );
}
