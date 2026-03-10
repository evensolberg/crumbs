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

export { EditorState, Compartment } from '@codemirror/state';

export {
  defaultKeymap,
  history,
  historyKeymap,
  indentWithTab,
  deleteLine,
  moveLineUp,
  moveLineDown,
} from '@codemirror/commands';

export {
  closeBrackets,
  closeBracketsKeymap,
} from '@codemirror/autocomplete';

export {
  search,
  searchKeymap,
  highlightSelectionMatches,
  openSearchPanel,
} from '@codemirror/search';

export {
  syntaxHighlighting,
  defaultHighlightStyle,
  HighlightStyle,
} from '@codemirror/language';

export { tags } from '@lezer/highlight';

export { markdown } from '@codemirror/lang-markdown';
