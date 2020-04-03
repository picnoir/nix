programs += error-demo

error-demo_DIR := $(d)

error-demo_SOURCES := \
  $(wildcard $(d)/*.cc) \

error-demo_LIBS = libutil

error-demo_LDFLAGS = -pthread $(SODIUM_LIBS) $(EDITLINE_LIBS) $(BOOST_LDFLAGS) -lboost_context -lboost_thread -lboost_system
