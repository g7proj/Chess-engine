import subprocess
import threading
import queue
import time
from typing import Callable, Optional, List


class UCIClient:
    """
    Minimal UCI client to start an engine process, send commands and receive stdout lines.
    Callback signature: fn(line: str)
    """

    def __init__(self, engine_path: str, on_line: Optional[Callable[[str], None]] = None):
        self.engine_path = engine_path
        self.on_line = on_line
        self.process: Optional[subprocess.Popen] = None
        self._reader_thread: Optional[threading.Thread] = None
        self._stop_event = threading.Event()
        self._write_lock = threading.Lock()
        self._response_queue: "queue.Queue[str]" = queue.Queue()

    def start(self):
        if self.process is not None:
            return
        self.process = subprocess.Popen(
            [self.engine_path],
            stdin=subprocess.PIPE,
            stdout=subprocess.PIPE,
            stderr=subprocess.STDOUT,
            text=True,
            bufsize=1,
        )
        self._stop_event.clear()
        self._reader_thread = threading.Thread(target=self._read_loop, daemon=True)
        self._reader_thread.start()
        # Initialize UCI
        self.send_command("uci")
        self.send_command("isready")

    def _read_loop(self):
        assert self.process is not None
        for line in self.process.stdout:
            l = line.rstrip("\n")
            if self.on_line:
                try:
                    self.on_line(l)
                except Exception:
                    pass
            # always put line into response queue for synchronous callers
            try:
                self._response_queue.put_nowait(l)
            except Exception:
                pass
            if self._stop_event.is_set():
                break

    def send_command(self, cmd: str):
        if self.process is None or self.process.stdin is None:
            return
        with self._write_lock:
            try:
                self.process.stdin.write(cmd.strip() + "\n")
                self.process.stdin.flush()
            except Exception:
                pass

    def send_position(self, moves: List[str]):
        if not moves:
            self.send_command("position startpos")
        else:
            moves_str = " ".join(moves)
            self.send_command(f"position startpos moves {moves_str}")

    def go(self, depth: int = 10):
        self.send_command(f"go depth {depth}")

    def get_legal_moves(self, timeout: float = 1.0) -> List[str]:
        """
        Request legal moves from the engine via the custom 'listmoves' command.
        Returns a list of UCI move strings or an empty list on timeout/error.
        """
        # flush any pending responses
        while not self._response_queue.empty():
            try:
                self._response_queue.get_nowait()
            except Exception:
                break

        self.send_command("listmoves")
        end_time = time.time() + timeout
        collected: List[str] = []
        while time.time() < end_time:
            try:
                line = self._response_queue.get(timeout=0.05)
            except queue.Empty:
                continue
            if line.startswith("legalmoves"):
                parts = line.split()
                if len(parts) >= 2:
                    return parts[1:]
                else:
                    return []
            else:
                # ignore other lines but keep looping until timeout or expected response
                collected.append(line)
        return []

    def get_fen(self, timeout: float = 1.0) -> str:
        """
        Request current FEN from engine via custom 'showfen' command.
        Returns FEN string (without 'fen ' prefix) or empty string on timeout/error.
        """
        # flush queue
        while not self._response_queue.empty():
            try:
                self._response_queue.get_nowait()
            except Exception:
                break

        self.send_command("showfen")
        end_time = time.time() + timeout
        while time.time() < end_time:
            try:
                line = self._response_queue.get(timeout=0.05)
            except queue.Empty:
                continue
            if line.startswith("fen "):
                return line[4:].strip()
            # ignore other lines
        return ""

    def wait_ready(self, timeout: float = 1.0) -> bool:
        """
        Send 'isready' to the engine and wait for 'readyok'. Returns True if readyok received.
        """
        # flush queue
        while not self._response_queue.empty():
            try:
                self._response_queue.get_nowait()
            except Exception:
                break

        self.send_command("isready")
        end_time = time.time() + timeout
        while time.time() < end_time:
            try:
                line = self._response_queue.get(timeout=0.05)
            except queue.Empty:
                continue
            if line.strip() == "readyok":
                return True
            # ignore other lines
        return False

    def stop(self):
        self.send_command("stop")

    def quit(self):
        if self.process is None:
            return
        try:
            self.send_command("quit")
        except Exception:
            pass
        self._stop_event.set()
        time.sleep(0.05)
        try:
            if self.process.poll() is None:
                self.process.terminate()
        except Exception:
            pass


