document.addEventListener('DOMContentLoaded', () => {
  const status = document.getElementById('status');
  const appRoot = document.getElementById('app-root');
  const preventedLink = document.getElementById('prevented-link');
  const fragmentLink = document.getElementById('fragment-link');
  const summary = document.getElementById('summary');
  const action = document.getElementById('action');
  const actionPanel = document.getElementById('action-panel');
  const input = document.getElementById('text-input');
  const result = document.getElementById('result');
  const fragmentState = document.getElementById('fragment-state');

  appRoot.setAttribute('data-ready', 'true');
  status.textContent = `DOMContentLoaded executed by KRR V8 (${document.readyState})`;
  preventedLink.addEventListener('click', (event) => {
    event.preventDefault();
    status.textContent = 'Navigation prevented by KRR V8';
    status.style.backgroundColor = '#ffd1dc';
  });
  fragmentLink.addEventListener('click', () => {
    status.textContent = 'Same-document fragment requested by KRR V8';
    status.style.backgroundColor = '#c7d2fe';
  });
  summary.addEventListener('click', () => {
    status.textContent = 'Accordion toggled by KRR V8';
    status.style.backgroundColor = '#b8f2d0';
    summary.style.backgroundColor = '#b8f2d0';
    summary.style.color = '#17372a';
  });
  actionPanel.addEventListener('click', () => {
    result.textContent = 'Parent click listener must not run';
    result.style.backgroundColor = '#fca5a5';
  });
  action.addEventListener('click', (event) => {
    event.stopPropagation();
    result.textContent = 'Button click mutated the DOM';
    result.style.color = '#12513a';
    result.style.backgroundColor = '#ffe08a';
    status.textContent = 'Button action executed by KRR V8';
    status.style.backgroundColor = '#ffe08a';
    fragmentState.textContent = 'Button state preserved across fragment navigation';
  });
  input.addEventListener('input', (event) => {
    result.textContent = `Input event value: ${event.currentTarget.value}`;
    result.style.color = '#173f5f';
    result.style.backgroundColor = '#a7ddff';
    status.textContent = `Input event value: ${event.currentTarget.value}`;
    status.style.backgroundColor = '#a7ddff';
    fragmentState.textContent = `Input state preserved: ${event.currentTarget.value}`;
    fragmentState.style.backgroundColor = '#d6f5e3';
  });
});
