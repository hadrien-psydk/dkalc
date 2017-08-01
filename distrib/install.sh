#!/bin/bash

function HandleFail
{
	tput setaf 1
	echo "="
	echo "=   FAILURE"
	echo "="
	tput sgr0
	exit 1
}

function HandleSuccess
{
	tput setaf 2
	echo "="
	echo "=   SUCCESS"
	echo "="
	tput sgr0
	exit 0
}

# Set current directory to the one containing this script
cd "$(dirname "$0")"

pushd ..
cargo build --release || HandleFail
popd
LOCAL=""
sudo cp ../target/release/dkalc /usr/${LOCAL}bin/          || HandleFail
sudo mkdir -p /usr/${LOCAL}share/applications/             || HandleFail
sudo cp dkalc.desktop /usr/${LOCAL}share/applications/     || HandleFail
sudo mkdir -p /usr/${LOCAL}share/icons/hicolor/48x48/apps/ || HandleFail
sudo cp icon-48.png /usr/${LOCAL}share/icons/hicolor/48x48/apps/dkalc.png || HandleFail
sudo gtk-update-icon-cache /usr/${LOCAL}share/icons/hicolor     || HandleFail
HandleSuccess
