import('./pkg').catch(console.error);

  // let audio = new AudioContext();
  // // node that does fft
  // let node = audio.createAnalyser();
  //
  // // object that represents microphone
  // (async () => {
  //   // get microphone input(video or audio)
  //   let stream = await navigator.mediaDevices.getUserMedia({
  //     video: false,
  //     audio: {
  //       noiseSuppression: false,
  //       echoCancellation: false,
  //     }
  //   });
  //   // input audio node
  //   let audioNode = audio.createMediaStreamSource(stream);
  //   // now connect to analyzer
  //   audioNode.connect(node);
  //
  //   // can put this outside of async
  //   // want animation loop, use request animation frames
  //   requestAnimationFrame(drawLoop);
  // })();
  //
  // // create buffer
  //
  // // decide how big want buffer to be to hold fft
  // let kMaxFrequency = 20000;
  // let buffer = new Uint8Array(Math.floor(kMaxFrequency / audio.sampleRate * (node.fftSize / 2)));
  //
  // const drawLoop = () => {
  //   node.getByteFrequencyData(buffer);
  //   console.log(buffer);
  //   requestAnimationFrame(drawLoop);
  // }
