@dataclass
class Point:
    x: float = 0.0
    y: float = 0.0

    def __repr__(self):
        return f"Point({self.x}, {self.y})"
