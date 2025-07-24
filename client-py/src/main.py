from lib import Client, ClientMessage, Vector


def on_map_change(walls: list[tuple[Vector, Vector]]) -> ClientMessage | None:
    pass


def on_state_change(
    tanks: list[tuple[Vector, Vector, Vector]], bullets: list[tuple[Vector, Vector]]
) -> ClientMessage | None:
    return ClientMessage((50, 20), 1000, True)


client = Client(on_map_change, on_state_change)

client.run()
