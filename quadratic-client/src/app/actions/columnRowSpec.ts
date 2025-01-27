import { Action } from '@/app/actions/actions';
import { isEmbed } from '@/app/helpers/isEmbed';
import { quadraticCore } from '@/app/web-workers/quadraticCore/quadraticCore';
import { AddIcon, DeleteIcon } from '@/shared/components/Icons';
import { sheets } from '../grid/controller/Sheets';
import { ActionAvailabilityArgs, ActionSpec } from './actionsSpec';

const isColumnRowAvailable = ({ isAuthenticated }: ActionAvailabilityArgs) => {
  if (!sheets.sheet.cursor.hasOneColumnRowSelection(true)) return false;
  return !isEmbed && isAuthenticated;
};

const isColumnFinite = () => sheets.sheet.cursor.isSelectedColumnsFinite();
const isColumnRowAvailableAndColumnFinite = (args: ActionAvailabilityArgs) =>
  isColumnRowAvailable(args) && isColumnFinite();
const isRowFinite = () => sheets.sheet.cursor.isSelectedRowsFinite();
const isColumnRowAvailableAndRowFinite = (args: ActionAvailabilityArgs) => isColumnRowAvailable(args) && isRowFinite();

const insertColumnLeft: ActionSpec<void> = {
  label: 'Insert column to the left',
  isAvailable: isColumnRowAvailableAndColumnFinite,
  Icon: AddIcon,
  run: () =>
    quadraticCore.insertColumn(sheets.sheet.id, sheets.sheet.cursor.position.x, true, sheets.getCursorPosition()),
};

const insertColumnRight: ActionSpec<void> = {
  label: 'Insert column to the right',
  isAvailable: isColumnRowAvailableAndColumnFinite,
  Icon: AddIcon,
  run: () =>
    quadraticCore.insertColumn(sheets.sheet.id, sheets.sheet.cursor.position.x + 1, false, sheets.getCursorPosition()),
};

const deleteColumns: ActionSpec<void> = {
  label: 'Delete columns',
  isAvailable: ({ isAuthenticated }: ActionAvailabilityArgs) => !isEmbed && isAuthenticated && isColumnFinite(),
  Icon: DeleteIcon,
  run: () => {
    const columns = sheets.sheet.cursor.getSelectedColumns();
    quadraticCore.deleteColumns(sheets.sheet.id, columns, sheets.getCursorPosition());
  },
};

const insertRowAbove: ActionSpec<void> = {
  label: 'Insert row above',
  isAvailable: isColumnRowAvailableAndRowFinite,
  Icon: AddIcon,
  run: () => quadraticCore.insertRow(sheets.sheet.id, sheets.sheet.cursor.position.y, true, sheets.getCursorPosition()),
};

const insertRowBelow: ActionSpec<void> = {
  label: 'Insert row below',
  isAvailable: isColumnRowAvailableAndRowFinite,
  Icon: AddIcon,
  run: () =>
    quadraticCore.insertRow(sheets.sheet.id, sheets.sheet.cursor.position.y + 1, false, sheets.getCursorPosition()),
};

const deleteRows: ActionSpec<void> = {
  label: 'Delete rows',
  isAvailable: ({ isAuthenticated }: ActionAvailabilityArgs) => !isEmbed && isAuthenticated && isRowFinite(),
  Icon: DeleteIcon,
  run: () => {
    const rows = sheets.sheet.cursor.getSelectedRows();
    quadraticCore.deleteRows(sheets.sheet.id, rows, sheets.getCursorPosition());
  },
};

export const columnRowSpec = {
  [Action.InsertColumnLeft]: insertColumnLeft,
  [Action.InsertColumnRight]: insertColumnRight,
  [Action.DeleteColumn]: deleteColumns,
  [Action.InsertRowAbove]: insertRowAbove,
  [Action.InsertRowBelow]: insertRowBelow,
  [Action.DeleteRow]: deleteRows,
};
