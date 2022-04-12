
restart:
	dfx stop && dfx start --clean --background

deploy:
	dfx deploy
	
set_controller:
	dfx canister --wallet $$(dfx identity get-wallet) update-settings --all --controller  rrkah-fqaaa-aaaaa-aaaaq-cai

get_status:
	dfx canister call manage get_canister_status '(principal "rrkah-fqaaa-aaaaa-aaaaq-cai")'