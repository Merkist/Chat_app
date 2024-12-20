// Підключення до WebSocket-сервера
const ws = new WebSocket("ws://127.0.0.1:3030/chat");

// Елементи інтерфейсу
const chatDiv = document.getElementById("chat");
const messageInput = document.getElementById("message-input");
const sendBtn = document.getElementById("send-btn");

// Відображення повідомлень
ws.onmessage = (event) => {
    const msgDiv = document.createElement("div");
    msgDiv.textContent = event.data;
    chatDiv.appendChild(msgDiv);
    chatDiv.scrollTop = chatDiv.scrollHeight; // Прокрутка до останнього повідомлення
};

// Надсилання повідомлення
sendBtn.addEventListener("click", () => {
    const message = messageInput.value.trim();
    if (message) {
        ws.send(message); // Надсилаємо повідомлення через WebSocket
        messageInput.value = "";
    }
});

// Відправка повідомлення при натисканні Enter
messageInput.addEventListener("keypress", (e) => {
    if (e.key === "Enter") {
        sendBtn.click();
    }
});
