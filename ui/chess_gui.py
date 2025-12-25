from PySide6.QtWidgets import (
    QMainWindow,
    QWidget,
    QGridLayout,
    QPushButton,
    QVBoxLayout,
    QHBoxLayout,
    QTextEdit,
    QLabel,
    QLineEdit,
    QListWidget,
    QFileDialog,
)
from PySide6.QtCore import Qt, QSize, Signal, QTimer
from ui.uci_client import UCIClient
from typing import List, Optional

def coord_to_uci(row: int, col: int) -> str:
    # row 0 is top (rank 8), row 7 is bottom (rank 1)
    file_char = chr(ord("a") + col)
    rank_char = str(8 - row)
    return f"{file_char}{rank_char}"


# Unicode piece symbols
PIECE_SYMBOLS = {
    "wK": "\u2654",
    "wQ": "\u2655",
    "wR": "\u2656",
    "wB": "\u2657",
    "wN": "\u2658",
    "wP": "\u2659",
    "bK": "\u265A",
    "bQ": "\u265B",
    "bR": "\u265C",
    "bB": "\u265D",
    "bN": "\u265E",
    "bP": "\u265F",
    "": "",
}


class ChessGUI(QMainWindow):
    engine_output = Signal(str)
    def __init__(self):
        super().__init__()
        self.setWindowTitle("Chess - Human vs Human (UCI-aware)")
        self.resize(800, 600)

        self.engine: Optional[UCIClient] = None
        self.moves: List[str] = []
        self.selected: Optional[tuple[int, int]] = None
        self.side_to_move: str = "w"  # 'w' or 'b'
        self.legal_moves: List[str] = []
        self.legal_from: dict = {}
        self._current_destinations: List[tuple[int,int]] = []
        self._build_ui()
        # connect signal for thread-safe engine -> GUI updates
        self.engine_output.connect(self._on_engine_line)

    def _build_ui(self):
        central = QWidget()
        self.setCentralWidget(central)
        root_layout = QHBoxLayout()
        central.setLayout(root_layout)

        # Left: board with labels (9x9 grid)
        board_container = QWidget()
        board_grid = QGridLayout()
        board_container.setLayout(board_grid)
        board_grid.setSpacing(0)
        self.squares = [[None for _ in range(8)] for __ in range(8)]

        # rows with rank label + square buttons
        for r in range(8):
            for c in range(8):
                btn = QPushButton("")
                btn.setFixedSize(QSize(64, 64))
                # style background
                light = "#f0d9b5"
                dark = "#b58863"
                btn.setStyleSheet(f"background-color: {light if (r+c)%2==0 else dark}; font-size: 28px;")
                btn.clicked.connect(self._make_on_click(r, c))
                board_grid.addWidget(btn, r + 1, c)
                self.squares[r][c] = btn
            # optional right-side rank label
            rank_label_r = QLabel(str(8 - r))
            rank_label_r.setAlignment(Qt.AlignCenter)
            board_grid.addWidget(rank_label_r, r + 1, 9 - 0)  # placeholder, ignored by layout

        # bottom file labels
        for c in range(8):
            lbl = QLabel(chr(ord("a") + c))
            lbl.setAlignment(Qt.AlignCenter)
            board_grid.addWidget(lbl, 9 - 0, c)  # placeholder bottom row

        root_layout.addWidget(board_container, 0)

        # initialize starting pieces and update UI
        self._init_starting_position()

        # Right: controls and log
        right_widget = QWidget()
        right_layout = QVBoxLayout()
        right_widget.setLayout(right_layout)

        engine_row = QHBoxLayout()
        self.engine_path_edit = QLineEdit("engine/target/debug/engine")
        self.start_engine_btn = QPushButton("Start Engine")
        self.start_engine_btn.clicked.connect(self._on_start_engine)
        engine_row.addWidget(QLabel("Engine:"))
        engine_row.addWidget(self.engine_path_edit)
        engine_row.addWidget(self.start_engine_btn)
        right_layout.addLayout(engine_row)

        controls_row = QHBoxLayout()
        self.go_btn = QPushButton("Analyze (depth 8)")
        self.go_btn.clicked.connect(self._on_analyze)
        controls_row.addWidget(self.go_btn)
        self.export_pgn_btn = QPushButton("Export PGN")
        self.export_pgn_btn.clicked.connect(self._on_export_pgn)
        controls_row.addWidget(self.export_pgn_btn)
        right_layout.addLayout(controls_row)
        # Engine status / analysis info
        self.bestmove_label = QLabel("Bestmove: -")
        right_layout.addWidget(self.bestmove_label)
        self.score_label = QLabel("Score: -")
        right_layout.addWidget(self.score_label)
        self.nodes_label = QLabel("Nodes: -")
        right_layout.addWidget(self.nodes_label)
        self.time_label = QLabel("Time: -")
        right_layout.addWidget(self.time_label)
        self.nps_label = QLabel("NPS: -")
        right_layout.addWidget(self.nps_label)

        right_layout.addWidget(QLabel("Move history:"))
        self.history_list = QListWidget()
        right_layout.addWidget(self.history_list, 1)

        right_layout.addWidget(QLabel("Engine log:"))
        self.log = QTextEdit()
        self.log.setReadOnly(True)
        right_layout.addWidget(self.log, 1)

        root_layout.addWidget(right_widget, 1)

        # initialize legal moves structures
        self.legal_moves = []
        self.legal_from = {}

    def _make_on_click(self, r: int, c: int):
        def on_click():
            # If no selection, select this square (only if contains piece of side_to_move)
            if self.selected is None:
                symbol = self.board_pieces[r][c]
                if not symbol:
                    # Nothing has been selected, so nothing to do
                    return
                color = self._piece_color_at(r, c)
                if self.engine and color != self.side_to_move:
                    self.log.append("Not your turn to move that piece")
                    return
                # set selection and show possible destinations
                self.selected = (r, c)
                self._highlight_square(r, c, True)
                self._show_destinations((r, c))
                return

            # If clicked the same square again, deselect
            if self.selected == (r, c):
                self._highlight_square(r, c, False)
                self._clear_dest_highlights()
                self.selected = None
                return

            # Attempt move from selected -> clicked
            from_sq = self.selected
            to_sq = (r, c)
            self._highlight_square(from_sq[0], from_sq[1], False)
            self._clear_dest_highlights()
            self.selected = None
            uci_move = coord_to_uci(from_sq[0], from_sq[1]) + coord_to_uci(to_sq[0], to_sq[1])

            # If engine present, ask engine whether this move is legal using cached legal_moves
            legal = True
            if self.engine:
                if self.legal_moves:
                    legal = uci_move in self.legal_moves
                else:
                    # fallback: if we don't have cached legal moves, allow move
                    legal = True

            if not legal:
                self.log.append(f"Illegal move: {uci_move}")
                return

            # Apply move
            self.moves.append(uci_move)
            self.history_list.addItem(uci_move)
            self._update_board_ui(from_sq, to_sq)
            # toggle side to move
            self.side_to_move = "b" if self.side_to_move == "w" else "w"
            # Send new position to engine if present, wait for engine to process, then resync FEN
            if self.engine:
                try:
                    self.engine.send_position(self.moves)
                    # request analysis
                    self.engine.go(depth=8)
                    # wait until engine processed the position
                    if self.engine.wait_ready(timeout=1.0):
                        fen = self.engine.get_fen(timeout=0.5)
                        if fen:
                            self._set_board_from_fen(fen)
                except Exception:
                    # fallback: do nothing special
                    pass
                # refresh legal moves after engine updates
                self._update_legal_moves_from_engine()

        return on_click

    def _highlight_square(self, r: int, c: int, enable: bool):
        btn = self.squares[r][c]
        if enable:
            btn.setStyleSheet("background-color: yellow;")
        else:
            # restore original board color (match _build_ui colors)
            light = "#f0d9b5"
            dark = "#b58863"
            btn.setStyleSheet(f"background-color: {light if (r+c)%2==0 else dark}; font-size: 28px;")

    def _update_board_ui(self, from_sq, to_sq):
        fr, fc = from_sq
        tr, tc = to_sq
        # update internal board representation and button labels
        piece = self.board_pieces[fr][fc]
        self.board_pieces[fr][fc] = ""
        self.board_pieces[tr][tc] = piece
        self.squares[fr][fc].setText("")
        self.squares[tr][tc].setText(piece)
        # After a move that may involve special updates (castling, en-passant),
        # better to resync full board from engine if available.
        if self.engine:
            fen = self.engine.get_fen(timeout=0.5)
            if fen:
                self._set_board_from_fen(fen)
                # refresh legal moves according to new position
                self._update_legal_moves_from_engine()

    def _uci_to_coord(self, sq: str):
        # e.g. 'e2' -> (row, col)
        if len(sq) != 2:
            return None
        file_char = sq[0]
        rank_char = sq[1]
        col = ord(file_char) - ord("a")
        row = 8 - int(rank_char)
        if 0 <= row < 8 and 0 <= col < 8:
            return (row, col)
        return None

    def _piece_color_at(self, r: int, c: int) -> Optional[str]:
        """Return 'w' or 'b' if a piece exists at r,c, otherwise None."""
        sym = self.board_pieces[r][c]
        if not sym:
            return None
        try:
            code = ord(sym)
        except Exception:
            return None
        # white pieces U+2654..U+2659, black U+265A..U+265F
        if 0x2654 <= code <= 0x2659:
            return "w"
        if 0x265A <= code <= 0x265F:
            return "b"
        return None

    def _update_legal_moves_from_engine(self):
        """Query engine for legal moves and enable/disable squares accordingly."""
        if not self.engine:
            # enable all if no engine
            for r in range(8):
                for c in range(8):
                    self.squares[r][c].setEnabled(True)
            return

        try:
            moves = self.engine.get_legal_moves(timeout=1.0)
        except Exception:
            moves = []
        self.legal_moves = moves or []
        self.legal_from = {}
        for m in self.legal_moves:
            if len(m) >= 4:
                from_sq = m[0:2]
                to_sq = m[2:4]
                fcoord = self._uci_to_coord(from_sq)
                tcoord = self._uci_to_coord(to_sq)
                if fcoord:
                    self.legal_from.setdefault(fcoord, []).append(tcoord)

        # enable only squares that have pieces of side_to_move and that have at least one legal move
        for r in range(8):
            for c in range(8):
                color = self._piece_color_at(r, c)
                if color == self.side_to_move and (r, c) in self.legal_from:
                    self.squares[r][c].setEnabled(True)
                else:
                    self.squares[r][c].setEnabled(False)

    def _show_destinations(self, from_sq: tuple[int, int]):
        """Highlight possible destination squares for a selected piece."""
        self._clear_dest_highlights()
        self._current_destinations = []
        dests = self.legal_from.get(from_sq, [])
        if not dests:
            return
        for coord in dests:
            if coord:
                r, c = coord
                # highlight destination with greenish background and enable it so it can be clicked
                self.squares[r][c].setStyleSheet("background-color: #9acd32; font-size: 28px;")
                self.squares[r][c].setEnabled(True)
                self._current_destinations.append((r, c))

    def _clear_dest_highlights(self):
        # restore original styles for all squares
        for r in range(8):
            for c in range(8):
                light = "#f0d9b5"
                dark = "#b58863"
                self.squares[r][c].setStyleSheet(f"background-color: {light if (r+c)%2==0 else dark}; font-size: 28px;")
        # restore enabled/disabled state according to legal_from (source squares enabled, others disabled)
        for r in range(8):
            for c in range(8):
                if (r, c) in self.legal_from:
                    self.squares[r][c].setEnabled(True)
                else:
                    self.squares[r][c].setEnabled(False)
        # clear current destinations cache
        self._current_destinations = []

    def _init_starting_position(self):
        # Setup pieces using Unicode symbols; row 0 is top (black backrank)
        self.board_pieces = [["" for _ in range(8)] for __ in range(8)]
        # black
        self.board_pieces[0] = [
            PIECE_SYMBOLS["bR"],
            PIECE_SYMBOLS["bN"],
            PIECE_SYMBOLS["bB"],
            PIECE_SYMBOLS["bQ"],
            PIECE_SYMBOLS["bK"],
            PIECE_SYMBOLS["bB"],
            PIECE_SYMBOLS["bN"],
            PIECE_SYMBOLS["bR"],
        ]
        self.board_pieces[1] = [PIECE_SYMBOLS["bP"]] * 8
        # empty middle
        for r in range(2, 6):
            self.board_pieces[r] = ["" for _ in range(8)]
        # white
        self.board_pieces[6] = [PIECE_SYMBOLS["wP"]] * 8
        self.board_pieces[7] = [
            PIECE_SYMBOLS["wR"],
            PIECE_SYMBOLS["wN"],
            PIECE_SYMBOLS["wB"],
            PIECE_SYMBOLS["wQ"],
            PIECE_SYMBOLS["wK"],
            PIECE_SYMBOLS["wB"],
            PIECE_SYMBOLS["wN"],
            PIECE_SYMBOLS["wR"],
        ]
        # apply to buttons
        for r in range(8):
            for c in range(8):
                self.squares[r][c].setText(self.board_pieces[r][c])

    def _set_board_from_fen(self, fen: str):
        """Parse fen string and update board_pieces/buttons accordingly."""
        if not fen:
            return
        parts = fen.split()
        if not parts:
            return
        placement = parts[0]
        rows = placement.split('/')
        if len(rows) != 8:
            return
        # mapping fen char -> unicode symbol used in UI
        fen_map = {
            'K': PIECE_SYMBOLS["wK"],
            'Q': PIECE_SYMBOLS["wQ"],
            'R': PIECE_SYMBOLS["wR"],
            'B': PIECE_SYMBOLS["wB"],
            'N': PIECE_SYMBOLS["wN"],
            'P': PIECE_SYMBOLS["wP"],
            'k': PIECE_SYMBOLS["bK"],
            'q': PIECE_SYMBOLS["bQ"],
            'r': PIECE_SYMBOLS["bR"],
            'b': PIECE_SYMBOLS["bB"],
            'n': PIECE_SYMBOLS["bN"],
            'p': PIECE_SYMBOLS["bP"],
        }
        new_board = [["" for _ in range(8)] for __ in range(8)]
        for r_idx, row_str in enumerate(rows):
            file = 0
            for ch in row_str:
                if ch.isdigit():
                    n = int(ch)
                    for _ in range(n):
                        new_board[r_idx][file] = ""
                        file += 1
                else:
                    new_board[r_idx][file] = fen_map.get(ch, "")
                    file += 1
        # Update UI (note: engine FEN uses ranks from 8->1; our rows are 0..7 top->bottom so mapping matches)
        self.board_pieces = new_board
        for r in range(8):
            for c in range(8):
                self.squares[r][c].setText(self.board_pieces[r][c])

    def _on_start_engine(self):
        path = self.engine_path_edit.text().strip()
        if not path:
            self.log.append("Engine path empty")
            return
        if self.engine:
            try:
                self.engine.quit()
            except Exception:
                pass
        # use _emit_engine_line so engine output is routed through Qt Signal (thread-safe)
        self.engine = UCIClient(path, on_line=self._emit_engine_line)
        try:
            self.engine.start()
            self.log.append(f"Started engine: {path}")
            # new game
            self.moves = []
            self.history_list.clear()
            self.engine.send_command("ucinewgame")
            self.engine.send_command("isready")
            # Query legal moves shortly after engine start (give engine time to process)
            QTimer.singleShot(300, self._update_legal_moves_from_engine)
        except Exception as e:
            self.log.append(f"Failed to start engine: {e}")

    def _emit_engine_line(self, line: str):
        # Called from engine thread; emit signal to be handled in GUI thread
        try:
            self.engine_output.emit(line)
        except Exception:
            # if signal emission fails, fallback to appending raw (may be unsafe)
            self.log.append(line)

    def _on_engine_line(self, line: str):
        # This runs in the GUI thread via Signal => safe to update widgets
        try:
            if line.startswith("info"):
                info = self._parse_uci_info(line)
                # Update score if present
                if "score" in info:
                    stype, sval = info["score"]
                    if stype == "cp":
                        self.score_label.setText(f"Score: {sval} cp")
                    elif stype == "mate":
                        self.score_label.setText(f"Mate in {sval}")
                # Update PV if present
                if "pv" in info:
                    pv = info["pv"]
                    self.bestmove_label.setText(f"PV: {' '.join(pv)}")
                # nodes/time/nps
                if "nodes" in info:
                    self.nodes_label.setText(f"Nodes: {info['nodes']}")
                if "time" in info:
                    # show ms → convert to s with 2 decimals
                    ms = info["time"]
                    try:
                        s = float(ms) / 1000.0
                        self.time_label.setText(f"Time: {s:.2f}s")
                    except Exception:
                        self.time_label.setText(f"Time: {ms} ms")
                if "nps" in info:
                    self.nps_label.setText(f"NPS: {info['nps']}")
                # append full info line to log
                self.log.append(line)
                return
            if line.startswith("bestmove"):
                parts = line.split()
                bm = parts[1] if len(parts) > 1 else "(none)"
                self.bestmove_label.setText(f"Bestmove: {bm}")
                self.log.append(line)
                return
        except Exception:
            # fallback: always append the raw line
            pass
        # default: append raw line
        self.log.append(line)

    def _parse_uci_info(self, line: str):
        parts = line.split()
        result = {}
        i = 0
        while i < len(parts):
            p = parts[i]
            if p == "score" and i + 2 < len(parts):
                stype = parts[i + 1]
                try:
                    sval = int(parts[i + 2])
                except Exception:
                    sval = parts[i + 2]
                result["score"] = (stype, sval)
                i += 3
                continue
            if p == "pv":
                # pv goes until end (or until another known token; keep simple)
                result["pv"] = parts[i + 1 :]
                break
            if p == "nodes" and i + 1 < len(parts):
                try:
                    result["nodes"] = int(parts[i + 1])
                except Exception:
                    result["nodes"] = parts[i + 1]
                i += 2
                continue
            if p == "time" and i + 1 < len(parts):
                try:
                    result["time"] = int(parts[i + 1])
                except Exception:
                    result["time"] = parts[i + 1]
                i += 2
                continue
            if p == "nps" and i + 1 < len(parts):
                try:
                    result["nps"] = int(parts[i + 1])
                except Exception:
                    result["nps"] = parts[i + 1]
                i += 2
                continue
            i += 1
        return result

    def _on_analyze(self):
        if not self.engine:
            self.log.append("Engine not running")
            return
        self.engine.send_position(self.moves)
        self.engine.go(depth=8)

    def _on_export_pgn(self):
        path, _ = QFileDialog.getSaveFileName(self, "Export PGN", "", "PGN files (*.pgn);;All files (*.*)")
        if not path:
            return
        # Very simple pgn: moves separated by spaces (no move numbers or headers)
        try:
            with open(path, "w", encoding="utf-8") as fh:
                fh.write(" ".join(self.moves))
            self.log.append(f"Exported PGN-like moves to {path}")
        except Exception as e:
            self.log.append(f"Failed to export PGN: {e}")

