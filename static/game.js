const Direction = {
    UP: 'up',
    DOWN: 'down',
    LEFT: 'left',
    RIGHT: 'right'
}

const KeyCode = {
    ArrowUp: 'ArrowUp',
    ArrowDown: 'ArrowDown',
    ArrowRight: 'ArrowRight',
    ArrowLeft: 'ArrowLeft',
}

const colors = {
    background: 'black',
    player1: 'red',
    player2: 'blue',
}

class Renderer {
    resolution = {
        width: 80,
        height: 60
    };
    client = {
        width: 640,
        height: 480,
    }

    pixel = {
        width: this.client.width / this.resolution.width,
        height: this.client.height / this.resolution.height
    }

    constructor() {
        const canvas = document.getElementById('canvas');
        canvas.width = this.client.width;
        canvas.height = this.client.height;

        this.ctx = canvas.getContext("2d");
    }

    drawPoint(x, y, color) {
        this.ctx.save();
        this.ctx.beginPath();

        this.ctx.fillStyle = color;
        this.ctx.strokeStyle = 'white';

        const topLeft = {
            x: x * this.pixel.width,
            y: y * this.pixel.height,
        }


        this.ctx.rect(topLeft.x, topLeft.y, this.pixel.height, this.pixel.width);

        this.ctx.fill();
        this.ctx.stroke();

        this.ctx.restore();
    }

    clear() {
        this.ctx.fillStyle = 'green';
        this.ctx.clearRect(0, 0, this.client.width, this.client.height)
    }
}

class Snake {
    constructor(renderer, color) {
        this.renderer = renderer;
        this.color = color;
    }

    update(body) {
        this.body = body;
    }

    draw() {
        this.body.forEach(point => {
            this.renderer.drawPoint(point.x, point.y, this.color)
        })
    }
}


class Connection {
    _onMessage = () => {
    }

    initWebSocket() {
        this.webSocket = new WebSocket('ws://localhost:8080/ws/');

        this.webSocket.onmessage = ((event) => {
            const data = JSON.parse(event.data);
            this._onMessage(data);
        })
    }

    onMessage(cb) {
        this._onMessage = cb;
    }

    send(data) {
        this.webSocket.send(data)
    }

    connect(gameId) {
        this.webSocket.send(`/connect ${gameId}`);
    }

    stop() {
        this.webSocket.send("/stop");
    }

    constructor() {
        this.initWebSocket();
    }
}

class Game {
    constructor() {
        this.connection = new Connection();
        this.renderer = new Renderer();
        this.snakes = [
            new Snake(this.renderer, colors.player1),
            new Snake(this.renderer, colors.player2)
        ]

        this.initDom();
        this.initEventListeners();
        this.getGameIdFromUrl();

        this.connection.onMessage(this.handleMessage.bind(this))
    }

    getGameIdFromUrl() {
        let params = (new URL(document.location)).searchParams;
        let gameId = params.get("gameId");
        console.log(gameId)

        this.setGameId(gameId)
    }

    setGameId(gameId) {
        this.gameId = gameId;

        const url = new URL(window.location.href);
        url.searchParams.append('gameId', gameId);
        this.ui.gameId.href = url;
    }

    handleMessage(data) {
        if (data.game_session_id) {
            this.setGameId(data.game_session_id)
        }

        if (data.players) {
            this.tick(data);
        }
    }

    initDom() {
        this.ui = {};
        this.ui.gameId = document.getElementById('game-id');
    }

    initEventListeners() {
        const connectButton = document.getElementById('connect');
        const disconnectButton = document.getElementById('stop');
        const initWsButton = document.getElementById('init-ws');

        connectButton.addEventListener('click', () => {
            this.connection.connect(this.gameId);
        })

        disconnectButton.addEventListener('click', () => {
            this.connection.stop();
        })

        initWsButton.addEventListener('click', () => {
            this.connection.initWebSocket();
        })


        window.addEventListener("keydown", (event) => {
            const eventMap = {
                [KeyCode.ArrowLeft]: Direction.LEFT,
                [KeyCode.ArrowRight]: Direction.RIGHT,
                [KeyCode.ArrowUp]: Direction.UP,
                [KeyCode.ArrowDown]: Direction.DOWN,
            };

            this.connection.send(`/direction ${eventMap[event.key]}`);
        });
    }

    tick(data) {
        this.renderer.clear();
        this.snakes.forEach((snake, i) => {
            snake.update(data.players?.[i].body)
            snake.draw();
        })
    }
}

new Game();

