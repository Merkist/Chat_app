// Ініціалізація WebSocket з чатом
const chatId = document.getElementById("chatId").value;
const userId = document.getElementById("userId").value;

const socket = new WebSocket(`ws://127.0.0.1:3030/ws/chat/${chatId}`);

// Слухаємо повідомлення від сервера
socket.onmessage = function (event) {
    const message = event.data;
    displayMessage(message);
};

document.getElementById("sendButton").addEventListener("click", function () {
    const content = document.getElementById("messageInput").value;
    if (content.trim()) {
        // Надсилаємо повідомлення на сервер
        const message = {
            user_id: userId, // Використовуємо динамічний userId
            chat_id: chatId, // Використовуємо динамічний chatId
            content: content,
        };
        socket.send(JSON.stringify(message)); // Надсилаємо повідомлення
        document.getElementById("messageInput").value = "";
    }
});

function displayMessage(message) {
    const chatBox = document.getElementById("chatBox");
    const messageElement = document.createElement("div");
    messageElement.textContent = `${message.user_id}: ${message.content}`;
    chatBox.appendChild(messageElement);
}
