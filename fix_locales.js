const fs = require('fs');

const translations = {
  "en": { filter: "Filter (Regex)...", name: "Name", newName: "New Name" },
  "ja": { filter: "フィルター (正規表現)...", name: "名前", newName: "新しい名前" },
  "de": { filter: "Filtern (Regex)...", name: "Name", newName: "Neuer Name" },
  "es": { filter: "Filtrar (Expresión regular)...", name: "Nombre", newName: "Nuevo nombre" },
  "fr": { filter: "Filtrer (Regex)...", name: "Nom", newName: "Nouveau nom" },
  "it": { filter: "Filtra (Regex)...", name: "Nome", newName: "Nuovo nome" },
  "ko": { filter: "필터 (정규식)...", name: "이름", newName: "새 이름" },
  "pt": { filter: "Filtrar (Expressão regular)...", name: "Nome", newName: "Novo nome" },
  "zh-CN": { filter: "过滤 (正则表达式)...", name: "名称", newName: "新名称" },
  "zh-TW": { filter: "篩選 (正則表達式)...", name: "名稱", newName: "新名稱" }
};

for (const [lang, trans] of Object.entries(translations)) {
  const path = `./crates/katana-ui/locales/${lang}.json`;
  const data = JSON.parse(fs.readFileSync(path, 'utf-8'));
  data.workspace.filter_regex_hint = trans.filter;
  data.dialog.name_hint = trans.name;
  data.dialog.new_name_hint = trans.newName;
  fs.writeFileSync(path, JSON.stringify(data, null, 2) + "\n");
}
