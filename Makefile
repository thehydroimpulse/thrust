all:
	echo Nothing to do...

docs:
	cargo doc
	in-dir ./target/doc fix-perms
	rscp ./target/doc/* gopher:~/www/burntsushi.net/rustdoc/

push:
	git push origin master
	git push github master
