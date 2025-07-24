from os import getenv
from socket import SOCK_DGRAM, SocketType, socket
from time import sleep
from collections.abc import Callable
from struct import unpack, pack


class Vector:
    __x: float
    __y: float

    def __init__(self, x: float, y: float):
        self.__x = x
        self.__y = y

    def __str__(self) -> str:
        return "< " + str(self.__x) + ", " + str(self.__y) + " >"


def secret_to_bytes(secret: int) -> bytes:
    res = bytes()
    for i in range(8):
        res = pack(">H", secret % 0x10000) + res
        secret = secret >> 16
    return res


class ClientMessage:
    track_acceleration_target: tuple[float, float]
    turret_acceleration_target: float
    shoot: bool

    def __init__(
        self,
        track_acceleration_target: tuple[float, float],
        turret_acceleration_target: float,
        shoot: bool,
    ) -> None:
        self.track_acceleration_target = track_acceleration_target
        self.turret_acceleration_target = turret_acceleration_target
        self.shoot = shoot

    def to_bytes(self, self_port, secret) -> bytes:
        return (
            b"\01"
            + pack(">H", self_port)
            + secret_to_bytes(secret)
            + pack(">f", self.track_acceleration_target[0])
            + pack(">f", self.track_acceleration_target[1])
            + pack(">f", self.turret_acceleration_target)
            + pack("?", self.shoot)
        )

    def __str__(self) -> str:
        if self.shoot:
            return (
                "{ "
                + str(self.track_acceleration_target)
                + ", "
                + str(self.turret_acceleration_target)
                + ", shoot }"
            )
        return (
            "{ "
            + str(self.track_acceleration_target)
            + ", "
            + str(self.turret_acceleration_target)
            + " }"
        )


def parse_wall(bytes: bytes) -> tuple[Vector, Vector] | None:
    assert len(bytes) == 16
    return (
        Vector(unpack("f", bytes[0:4])[0], unpack("f", bytes[4:8])[0]),
        Vector(unpack("f", bytes[8:12])[0], unpack("f", bytes[12:16])[0]),
    )


def parse_map_change(bytes: bytes) -> list[tuple[Vector, Vector]] | None:
    wall_count = bytes[0]
    walls = []
    for i in range(wall_count):
        walls.append(parse_wall(bytes[1 + 16 * i : 17 + 16 * i]))
    return walls


def parse_tank(bytes: bytes):
    assert len(bytes) == 24
    return (
        Vector(unpack("f", bytes[0:4])[0], unpack("f", bytes[4:8])[0]),
        Vector(unpack("f", bytes[8:12])[0], unpack("f", bytes[12:16])[0]),
        Vector(unpack("f", bytes[16:20])[0], unpack("f", bytes[20:24])[0]),
    )


def parse_bullet(bytes: bytes):
    assert len(bytes) == 16
    return (
        Vector(unpack("f", bytes[0:4])[0], unpack("f", bytes[4:8])[0]),
        Vector(unpack("f", bytes[8:12])[0], unpack("f", bytes[12:16])[0]),
    )


def parse_state_change(
    bytes: bytes,
) -> tuple[list[tuple[Vector, Vector, Vector]], list[tuple[Vector, Vector]]] | None:
    tank_count = bytes[0]
    tanks = []
    for i in range(tank_count):
        tanks.append(parse_tank(bytes[1 + i * 24 : 25 + i * 24]))
    bullet_count = bytes[1 + tank_count * 24]
    bullets = []
    for i in range(bullet_count):
        bullets.append(
            parse_bullet(
                bytes[1 + tank_count * 24 + i * 16 : 17 + tank_count * 24 + i * 16]
            )
        )
    return (tanks, bullets)


def parse_secret(incoming: bytes) -> int:
    secret = 0
    for i in incoming:
        secret = 256 * secret + i
    return secret


class Client:
    socket: SocketType
    self_port: int
    secret: int = 0
    __on_map_change: Callable[[list[tuple[Vector, Vector]]], ClientMessage | None]
    __on_state_change: Callable[
        [list[tuple[Vector, Vector, Vector]], list[tuple[Vector, Vector]]],
        ClientMessage | None,
    ]

    def __init__(self, map_change_callback, state_change_callback):
        self.socket = socket(type=SOCK_DGRAM)
        self.self_port = int(getenv("SELF-PORT") or "4001")
        self.server_port = int(getenv("SERVER") or "4000")
        self.socket.bind(("127.0.0.1", self.self_port))
        self.__on_map_change = map_change_callback
        self.__on_state_change = state_change_callback

    def _connect(self):
        msg = [0, int(self.self_port / 256) % 256, self.self_port % 256]
        print(msg)
        self.socket.sendto(bytes(msg), ("127.0.0.1", self.server_port))

    def run(self):
        while True:
            print("Connecting...")
            self._connect()
            while incoming := self.socket.recv(4885):
                match incoming[0]:
                    case 0:  # map change
                        self.secret = parse_secret(incoming[1:17])
                        walls = parse_map_change(incoming[17:])
                        if walls:
                            msg = self.__on_map_change(walls)
                            if msg:
                                self.socket.sendto(
                                    msg.to_bytes(self.self_port, self.secret),
                                    ("127.0.0.1", self.server_port),
                                )
                    case 1:  # state
                        self.secret = parse_secret(incoming[1:17])
                        parsed = parse_state_change(incoming[17:])
                        if parsed:
                            (tanks, bullets) = parsed
                            msg = self.__on_state_change(tanks, bullets)
                            if msg:
                                print(
                                    ["pos: " + str(a) + ", dir: " + str(b) + ", tur: " + str(c) for (a, b, c) in tanks],
                                    ["pos: " + str(a) + ", dir: " + str(b) for (a, b) in bullets],
                                    "->",
                                    str(msg),
                                    ",",
                                    self.self_port,
                                    ",",
                                    self.secret,
                                    "-->",
                                    msg.to_bytes(self.self_port, self.secret),
                                )
                                self.socket.sendto(
                                    msg.to_bytes(self.self_port, self.secret),
                                    ("127.0.0.1", self.server_port),
                                )
                    case 2:  # disconnect
                        self._connect()
                    case _:
                        print("wut??", incoming)
            sleep(0.1)
