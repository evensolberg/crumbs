// CodeMirror 6 bundle entry — exports everything used by crumbs-gui/main.js.
// To rebuild: cd crumbs-gui && npm run build-cm6

export {
  EditorView,
  keymap,
  lineNumbers,
  highlightActiveLine,
  highlightActiveLineGutter,
  drawSelection,
  dropCursor,
  highlightSpecialChars,
  placeholder,
} from '@codemirror/view';

export { EditorState } from '@codemirror/state';

export {
  defaultKeymap,
  history,
  historyKeymap,
  indentWithTab,
} from '@codemirror/commands';

export {
  closeBrackets,
  closeBracketsKeymap,
} from '@codemirror/autocomplete';

export {
  search,
  searchKeymap,
  highlightSelectionMatches,
} from '@codemirror/search';

export {
  syntaxHighlighting,
  defaultHighlightStyle,
} from '@codemirror/language';

export { markdown } from '@codemirror/lang-markdown';
