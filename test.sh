#!/bin/bash

# Script de test pour nwidgets
# Ce script lance nwidgets et envoie des notifications de test

echo "🚀 Lancement de nwidgets..."
cargo run &
NWIDGETS_PID=$!

echo "⏳ Attente de 3 secondes pour que nwidgets démarre..."
sleep 3

echo "📨 Envoi de notifications de test..."

# Notification normale
notify-send "Test 1" "Ceci est une notification de test normale" -u normal

sleep 1

# Notification critique
notify-send "Test 2" "Ceci est une notification critique" -u critical

sleep 1

# Notification avec body plus long
notify-send "Test 3" "Ceci est une notification avec un body plus long pour tester l'affichage" -u low

echo "✅ Notifications envoyées!"
echo ""
echo "📋 Instructions de test:"
echo "1. Appuyez sur CapsLock pour tester l'OSD CapsLock"
echo "2. Changez le volume avec vos touches multimédia pour tester l'OSD volume"
echo "3. Changez de workspace Hyprland pour tester la mise à jour du panel"
echo ""
echo "Pour arrêter nwidgets, appuyez sur Ctrl+C"

wait $NWIDGETS_PID
