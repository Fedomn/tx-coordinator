cmd =
ifeq "$(docker-compose)" ""
  cmd = lima nerdctl compose
endif

db:
	cd docker && $(cmd) -f ./docker-compose.yml up
