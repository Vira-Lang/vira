#!/bin/bash

# Kolory ANSI dla ładnego CLI
GREEN="\e[32m"
YELLOW="\e[33m"
BLUE="\e[34m"
RED="\e[31m"
RESET="\e[0m"

# Funkcja do wyświetlania spinnera podczas operacji
spinner() {
    local pid=$1
    local delay=0.1
    local spinstr='⠏⠇⠧⠦⠴⠼⠸⠹⠙⠋'
    while [ "$(ps a | awk '{print $1}' | grep $pid)" ]; do
        local temp=${spinstr#?}
        printf " [%c]  " "$spinstr"
        local spinstr=$temp${spinstr%"$temp"}
        sleep $delay
        printf "\b\b\b\b\b\b"
    done
    printf "    \b\b\b\b"
}

# Funkcja do wyświetlania komunikatów sukcesu
success() {
    echo -e "${GREEN}✔ $1${RESET}"
}

# Funkcja do wyświetlania komunikatów info
info() {
    echo -e "${BLUE}ℹ $1${RESET}"
}

# Funkcja do wyświetlania błędów
error() {
    echo -e "${RED}✖ $1${RESET}"
}

# Główny nagłówek
echo -e "${YELLOW}======================================"
echo -e "  Instalator Vira v0.0.1"
echo -e "======================================${RESET}"
echo ""

# Tworzenie katalogu ~/.vira
info "Tworzenie katalogu ~/.vira..."
mkdir -p ~/.vira && success "Katalog ~/.vira utworzony." || { error "Błąd tworzenia katalogu."; exit 1; }

# Tworzenie katalogu ~/.vira/bin
info "Tworzenie katalogu ~/.vira/bin..."
mkdir -p ~/.vira/bin && success "Katalog ~/.vira/bin utworzony." || { error "Błąd tworzenia katalogu."; exit 1; }

# Pobieranie plików do ~/.vira/bin
files=(
    "vira-compiler"
    "vira-packages"
    "vira-parser_lexer"
)

for file in "${files[@]}"; do
    url="https://github.com/Vira-Lang/vira/releases/download/v0.0.1/$file"
    info "Pobieranie $file..."
    (curl -sL "$url" -o ~/.vira/bin/$file) &
    spinner $!
    if [ $? -eq 0 ]; then
        success "$file pobrany."
        info "Nadawanie uprawnień +x dla $file..."
        chmod +x ~/.vira/bin/$file && success "Uprawnienia nadane." || { error "Błąd nadawania uprawnień."; exit 1; }
    else
        error "Błąd pobierania $file."
        exit 1
    fi
done

# Pobieranie vira do ~/.local/bin
info "Tworzenie katalogu ~/.local/bin jeśli nie istnieje..."
mkdir -p ~/.local/bin && success "Katalog ~/.local/bin gotowy." || { error "Błąd tworzenia katalogu."; exit 1; }

url="https://github.com/Vira-Lang/vira/releases/download/v0.0.1/vira"
info "Pobieranie vira..."
(curl -sL "$url" -o ~/.local/bin/vira) &
spinner $!
if [ $? -eq 0 ]; then
    success "vira pobrany."
    info "Nadawanie uprawnień +x dla vira..."
    chmod +x ~/.local/bin/vira && success "Uprawnienia nadane." || { error "Błąd nadawania uprawnień."; exit 1; }
else
    error "Błąd pobierania vira."
    exit 1
fi

echo ""
echo -e "${YELLOW}======================================"
echo -e "  Instalacja zakończona sukcesem!"
echo -e "  Dodaj ~/.local/bin do PATH jeśli potrzeba."
echo -e "======================================${RESET}"
