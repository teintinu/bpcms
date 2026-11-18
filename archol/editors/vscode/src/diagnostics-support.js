'use strict';

const OUTSIDE_APPLICATION_CODE = 'W_OUTSIDE_APPLICATION';

/** True when the LSP marked this document as outside the current application. */
function hasOutsideMarker(list) {
  return Array.isArray(list)
    && list.some((item) => String(item?.code) === OUTSIDE_APPLICATION_CODE);
}

module.exports = { OUTSIDE_APPLICATION_CODE, hasOutsideMarker };
