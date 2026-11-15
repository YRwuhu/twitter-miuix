const ICONS: Record<string, string> = {
  home: '<path d="M3.2 10.6 12 3.2l8.8 7.4V20a1.4 1.4 0 0 1-1.4 1.4h-5V15h-4.8v6.4h-5A1.4 1.4 0 0 1 3.2 20z"/>',
  homeFill:
    '<path fill="currentColor" stroke="none" d="M12 2.5 1.9 11a1.3 1.3 0 0 0-.5 1v8.6A1.9 1.9 0 0 0 3.3 22h5.3v-6.3a3.4 3.4 0 0 1 6.8 0V22h5.3a1.9 1.9 0 0 0 1.9-1.9V12a1.3 1.3 0 0 0-.5-1z"/>',
  search: '<circle cx="10.6" cy="10.6" r="6.6"/><path d="m15.5 15.5 5 5"/>',
  searchFill:
    '<path fill="currentColor" stroke="none" d="M10.6 3.2a7.4 7.4 0 0 1 5.8 11.9l4.7 4.7a1.1 1.1 0 0 1-1.6 1.6l-4.7-4.7A7.4 7.4 0 1 1 10.6 3.2m0 2.4a5 5 0 1 0 0 10 5 5 0 0 0 0-10"/>',
  mail: '<rect x="2.8" y="5.2" width="18.4" height="13.6" rx="2.6"/><path d="m3.6 7.4 8.4 6 8.4-6"/>',
  mailFill:
    '<path fill="currentColor" stroke="none" d="M4.9 4.6h14.2a2.4 2.4 0 0 1 2.4 2.4v10a2.4 2.4 0 0 1-2.4 2.4H4.9a2.4 2.4 0 0 1-2.4-2.4V7a2.4 2.4 0 0 1 2.4-2.4m.3 4.1 6.3 4.5a1 1 0 0 0 1 0l6.3-4.5-.7-1.1-6.1 4.3L5.9 7.6z"/>',
  user: '<circle cx="12" cy="8.1" r="3.9"/><path d="M4.4 21a7.6 7.6 0 0 1 15.2 0"/>',
  userFill:
    '<path fill="currentColor" stroke="none" d="M12 2.2a5.3 5.3 0 1 1 0 10.6 5.3 5.3 0 0 1 0-10.6m0 12.7c4.6 0 8.4 2.7 8.4 6v1.3H3.6V20.9c0-3.3 3.8-6 8.4-6"/>',
  edit: '<path d="M4 20h4.3L19 9.3a2.6 2.6 0 0 0-3.7-3.7L4.6 16.4z"/><path d="m14.4 6.6 3.4 3.4"/>',
  heart: '<path d="M12 20.6C9.8 19.3 3 14.8 3 9.9A4.9 4.9 0 0 1 12 6.6a4.9 4.9 0 0 1 9 3.3c0 4.9-6.8 9.4-9 10.7z"/>',
  heartFill:
    '<path fill="currentColor" stroke="none" d="M12 21c-2.5-1.5-10-6.5-10-11.8A5.9 5.9 0 0 1 12 5.1a5.9 5.9 0 0 1 10 4.1C22 14.5 14.5 19.5 12 21"/>',
  comment: '<path d="M20.6 11.6a8.2 8.2 0 0 1-12.1 7.2L4 20.3l1.5-4.4a8.2 8.2 0 1 1 15.1-4.3z"/>',
  repost:
    '<path d="M6.8 7.6h8.6a3.4 3.4 0 0 1 3.4 3.4v1.6"/><path d="m14.8 4.6 3 3-3 3"/><path d="M17.2 16.4H8.6a3.4 3.4 0 0 1-3.4-3.4V11.4"/><path d="m9.2 19.4-3-3 3-3"/>',
  bookmark: '<path d="M6.2 3.8h11.6v17l-5.8-4.2-5.8 4.2z"/>',
  bookmarkFill: '<path fill="currentColor" stroke="none" d="M5.6 3.2h12.8v18l-6.4-4.6-6.4 4.6z"/>',
  share:
    '<path d="M12 3v12"/><path d="m7.6 7.4 4.4-4.4 4.4 4.4"/><path d="M5 12.6v5.9a2.5 2.5 0 0 0 2.5 2.5h9a2.5 2.5 0 0 0 2.5-2.5v-5.9"/>',
  more: '<circle cx="5" cy="12" r="1.7" fill="currentColor" stroke="none"/><circle cx="12" cy="12" r="1.7" fill="currentColor" stroke="none"/><circle cx="19" cy="12" r="1.7" fill="currentColor" stroke="none"/>',
  bell: '<path d="M18.2 9.2a6.2 6.2 0 1 0-12.4 0c0 5.2-2.3 6.4-2.3 6.4h17s-2.3-1.2-2.3-6.4"/><path d="M10.3 20a2.2 2.2 0 0 0 3.4 0"/>',
  bellFill:
    '<path fill="currentColor" stroke="none" d="M12 2.2a6.7 6.7 0 0 1 6.7 6.7c0 5.1 2 6.2 2 7.4 0 .7-.6 1.1-1.4 1.1H4.7c-.8 0-1.4-.4-1.4-1.1 0-1.2 2-2.3 2-7.4A6.7 6.7 0 0 1 12 2.2m0 19.2a2.7 2.7 0 0 1-2.5-1.8h5a2.7 2.7 0 0 1-2.5 1.8"/>',
  settings:
    '<path d="M4 7h9.2"/><path d="M18.4 7H20"/><circle cx="15.8" cy="7" r="2.2"/><path d="M4 17h1.4"/><path d="M10.6 17H20"/><circle cx="8.2" cy="17" r="2.2"/>',
  arrowLeft: '<path d="M15 4.6 7.6 12l7.4 7.4"/>',
  close: '<path d="m6.2 6.2 11.6 11.6"/><path d="M17.8 6.2 6.2 17.8"/>',
  image: '<rect x="3" y="4.2" width="18" height="15.6" rx="3"/><circle cx="8.6" cy="9.6" r="1.6"/><path d="m4.2 17.4 5-5 4.2 4.2 2.8-2.6 3.6 3.4"/>',
  sun: '<circle cx="12" cy="12" r="4.2"/><path d="M12 2.2v2.2M12 19.6v2.2M2.2 12h2.2M19.6 12h2.2M5.1 5.1l1.6 1.6M17.3 17.3l1.6 1.6M18.9 5.1l-1.6 1.6M6.7 17.3l-1.6 1.6"/>',
  moon: '<path d="M20.4 14.6A8.6 8.6 0 0 1 9.4 3.6a8.7 8.7 0 1 0 11 11z"/>',
  check: '<path d="m5 12.6 4.6 4.6L19 7.4"/>',
  globe:
    '<circle cx="12" cy="12" r="9"/><path d="M3.2 12h17.6"/><path d="M12 3a14 14 0 0 1 0 18 14 14 0 0 1 0-18z"/>',
}

export default ICONS