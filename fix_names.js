const fs = require('fs');

const pathDE = './crates/katana-ui/locales/de.json';
const pathIT = './crates/katana-ui/locales/it.json';
const pathPT = './crates/katana-ui/locales/pt.json';

let de = JSON.parse(fs.readFileSync(pathDE, 'utf8'));
de.dialog.name_hint = "Dateiname";
fs.writeFileSync(pathDE, JSON.stringify(de, null, 2) + "\n");

let it = JSON.parse(fs.readFileSync(pathIT, 'utf8'));
it.dialog.name_hint = "Nome del file";
fs.writeFileSync(pathIT, JSON.stringify(it, null, 2) + "\n");

let pt = JSON.parse(fs.readFileSync(pathPT, 'utf8'));
pt.dialog.name_hint = "Nome do arquivo";
fs.writeFileSync(pathPT, JSON.stringify(pt, null, 2) + "\n");
