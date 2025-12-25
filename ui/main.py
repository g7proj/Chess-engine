import sys

from PySide6.QtWidgets import QApplication

from ui.chess_gui import ChessGUI

# To run the GUI, use the following command:
# python -m ui.main

def main():
    app = QApplication(sys.argv)
    window = ChessGUI()
    window.show()
    sys.exit(app.exec())


if __name__ == "__main__":
    main()
