identity := "dzhx6-f63tz-aslp6-xxyzd-pknwt-lxpho-q2wsx-pvwwd-v3nq6-75ek5-rqe"
.PHONY : restart deploy set_controller get_status add_user get_user_info add_group remove_group add_project add_group_member remove_group_member
restart:
	dfx stop && dfx start --clean --background

deploy:
	dfx deploy
	
set_controller:
	dfx canister --wallet $$(dfx identity get-wallet) update-settings --all --controller  rrkah-fqaaa-aaaaa-aaaaq-cai

get_status:
	dfx canister call manage get_canister_status '(principal "rrkah-fqaaa-aaaaa-aaaaq-cai")'

add_user:
	dfx canister call manage add_user '("test1",variant { Public})'

get_user_info:
	dfx canister call manage get_user_info '(principal $(identity))'

add_group:
	dfx canister call manage add_group '(record {id = 10;create_time=100;name="aaa";description="bbb";visibility=variant {Private};projects = vec {};members = vec { record{0 =  principal $(identity);1 = record { name = "bb";authority = variant {Write}; identity = principal $(identity)}}}})'

remove_group:
	dfx canister call manage remove_group '(10)'

add_project:
	dfx canister call manage add_project '(10,record {id=1;name="bbb";description="bbb";create_by = principal $(identity);create_time = 100;git_repo_url = "www.****.git";visibility = variant {Private};in_group = 10;members = vec {};canisters = vec {}})'

remove_project:
	dfx canister call manage remove_project '(10,1)'

add_group_member:
	dfx canister call manage add_group_member '(10, record {name="aaa"; authority = variant {Write};identity = principal $(identity);})'

remove_group_member:
	dfx canister call manage remove_group_member '(10,principal $(identity))'
	
test:
	make restart && make deploy && make add_user && make add_group && make add_project && make add_group_member && make remove_group_member
