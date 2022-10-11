interface UpdateEvent {
    ts_millis: number,
    state: State,
    media_url: string,
}

type State = "Playing" | "Paused";

const API_BASE_URL = "85.166.32.173:8080";
const WS_URL = "ws://" + API_BASE_URL + "/ws";
// const wsUrl = "ws://localhost:8080";

let player: HTMLVideoElement | null = null;
console.log("busihness");
// const buttons = {
// button-pause
// button-play
// button-set
// button-seek
// button-close
// }

let lastUpdate: UpdateEvent | null = null;
let lastUpdateAt: number = Date.now();
let requiredSeeks = 0;

document.addEventListener("DOMContentLoaded", () => {
    console.log("asdasdasdsa");
    player = document.getElementById("player") as HTMLVideoElement | null;
    if (player === null) {
        throw new Error("well shit sherlock, we got no video");
    }

    connect();

    document.getElementById("button-pause")!.onclick = onPauseClick;
    document.getElementById("button-play")!.onclick = onPlayClick;
    document.getElementById("button-seturl")!.onclick = onSetUrlClick;
    document.getElementById("button-seek")!.onclick = onSeekClick;
    document.getElementById("button-close")!.onclick = onCloseClick;

    player.onpause = () => {
        if (lastUpdate && lastUpdate.state === "Playing") {
            player?.play();
        }
    }

    player.onplay = () => {
        if (lastUpdate && lastUpdate.state === "Paused") {
            player?.pause();
        } else if (lastUpdate && lastUpdate.state === "Playing") {
            // fast seek to proper place
            const target = lastUpdate.ts_millis + (Date.now() - lastUpdateAt)
            seek(target / 1000);
        }
    }
})

function connect() {
    console.log("connectign to ws at", WS_URL)
    let ws = new WebSocket(WS_URL);

    ws.onmessage = (msg) => {
        let deserialized: UpdateEvent = JSON.parse(msg.data);
        console.log("[WS] Got update: ", deserialized)
        handleUpdate(deserialized);
    }

    ws.onclose = (err) => {
        console.error("ws was closed: ", err, "reconnecting in a little bit");
        delayedConnect();
    }
}

function handleUpdate(update: UpdateEvent) {
    if (!player) {
        throw new Error("we aint got no player chief!");
    }

    const deviation = (update.ts_millis / 1000) - player.currentTime;
    document.getElementById("debug-info")!.innerText = JSON.stringify(update) + "last dev " + deviation + ", seeks " + requiredSeeks;
    lastUpdate = update;
    lastUpdateAt = Date.now();

    console.log("player paused? ", player.paused);

    if (player.src !== update.media_url && update.media_url) {
        player.src = update.media_url;
    }

    // player.currentTime
    if (player.paused && update.state === "Playing") {
        // player.pause
        if (Math.abs(deviation) > 1) {
            seek(update.ts_millis / 1000);
        }
        player.play();
    } else if (!player.paused && update.state === "Paused") {
        player.pause();
        if (Math.abs(deviation) > 1) {
            seek(update.ts_millis / 1000);
        }
    }
    // else if (Math.abs((update.ts_millis / 1000) - player.currentTime) > 2) {
    //     if (!player.seeking) {
    //         console.log("Seeking, old time: ", player.currentTime, "new time", update.ts_millis / 1000)
    //         seek(update.ts_millis / 1000);
    //         requiredSeeks++;
    //     }
    // }
}

function seek(ts) {
    if (player?.fastSeek) {
        player.fastSeek(ts)
    } else {
        player!.currentTime = ts;
    }
}

document.addEventListener("keypress", (key) => {
    if (key.key === "a" || key.key === "A") {
        document.getElementById("admin-panel")!.hidden = false;
    }
})

async function delayedConnect() {
    sleep(5000);
    connect();
}

function sleep(ms) {
    return new Promise(resolve => setTimeout(resolve, ms));
}

async function issueApiCall(path: string, body?: any) {
    const headers = new Headers();
    const inputPassword = (document.getElementById("input-password")! as HTMLInputElement).value;
    headers.append("Authorization", inputPassword);
    if (body) {
        headers.append("Content-Type", "application/json");
    }

    const resp = await fetch("http://" + API_BASE_URL + path, {
        method: "POST",
        body: body ? JSON.stringify(body) : "",
        headers: headers,
    });
    if (resp.status !== 200) {
        alert(resp.statusText);
    }
}

function onPauseClick() {
    console.log("we clicked da onPauseClick");
    issueApiCall("/pause");
}
function onPlayClick() {
    console.log("we clicked da onPlayClick");
    issueApiCall("/unpause");
}
function onSetUrlClick() {
    console.log("we clicked da onSetUrlClick");
    const inputUrl = (document.getElementById("input-url")! as HTMLInputElement).value;
    issueApiCall("/change_media", {
        new_url: inputUrl,
    });
}
function onSeekClick() {
    console.log("we clicked da onSeekClick");
    const inputSeek = (document.getElementById("input-seek")! as HTMLInputElement).value;
    const split = inputSeek.split(":");

    let newTs = 0;
    if (split.length === 1) {
        newTs = parseInt(split[0]) * 1000;
    } else if (split.length === 2) {
        const mins = parseInt(split[0])
        const secs = parseInt(split[1])
        newTs = (mins * 60 * 1000) + (secs * 1000);
        console.log(mins, secs, newTs);
    }

    if (newTs !== 0) {
        issueApiCall("/seek", {
            new_ts_milliseconds: newTs,
        });
    }
}
function onCloseClick() {
    console.log("we clicked da onCloseClick");
    document.getElementById("admin-panel")!.hidden = true;
}