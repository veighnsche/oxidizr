import json
from datetime import datetime, timezone
from pathlib import Path
from typing import Optional


class JSONLLogger:
    def __init__(self, jsonl_path: Path, *, component: str = "runner", run_id: Optional[str] = None,
                 container_id: Optional[str] = None, distro: Optional[str] = None):
        self.path = Path(jsonl_path)
        self.path.parent.mkdir(parents=True, exist_ok=True)
        self.component = component
        self.run_id = run_id
        self.container_id = container_id
        self.distro = distro

    def event(self, *, stage: str, suite: Optional[str], cmd: Optional[str] = None,
              rc: Optional[int] = None, elapsed_ms: Optional[int] = None,
              level: str = "info", msg: Optional[str] = None, event: Optional[str] = None):
        rec = {
            "ts": datetime.now(timezone.utc).isoformat(),
            "component": self.component,
            "run_id": self.run_id,
            "container_id": self.container_id,
            "distro": self.distro,
            "level": level,
            "stage": stage,
            "suite": suite,
            "event": event,
            "cmd": cmd,
            "rc": rc,
            "duration_ms": elapsed_ms,
        }
        if msg is not None:
            rec["message"] = msg
        line = json.dumps(rec, ensure_ascii=False)
        with self.path.open("a", encoding="utf-8") as f:
            f.write(line + "\n")
