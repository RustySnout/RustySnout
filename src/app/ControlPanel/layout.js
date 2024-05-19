import styles from './layout.module.css';
import React, { useState, useEffect } from 'react';
import { invoke } from "@tauri-apps/api/tauri";

const Dropdown = ({ options, prompt, value, onChange }) => {
  return (
    <div className={styles.dropdown}>
      <select value={value} onChange={(e) => onChange(e.target.value)}>
        <option disabled value="">
          {prompt}
        </option>
        {options.map((option) => (
          <option key={option.value} value={option.value}>
            {option.label}
          </option>
        ))}
      </select>
    </div>
  );
};

const ControlPanel = () => {
  const [selectedInterface, setSelectedInterface] = useState('N / A'); 
  const [options, setOptions] = useState([]);
  const [intThrate, setinThRate] = useState(0);
  const [appThUprate, setAppThUprate] = useState(0);
  const [appThDownrate, setAppThDownrate] = useState(0);
  const [app, setApp] = useState('');


  const handleThInt = () => {
    console.log("Throttle Interface");
    invoke("interface_throttling_wrapper", { interface: selectedInterface, bandwidthLimit: parseInt(intThrate) }).then((res) => {
      console.log(res);
    }).catch((err) => {
      console.log(err);
    });
  }

  const handleThApp = () => {
    console.log("Throttle App");
    invoke("launch_throttled_app_wrapper", { app: app, upLimit: parseInt(appThUprate), downLimit: parseInt(appThDownrate) }).then((res) => {
      console.log(res);
    }).catch((err) => {
      console.log(err);
    });
  }

  const handleUpdateInterval = () => {
    console.log("Set Update Interval");
  }

  const handleRmTh = () => {
    console.log("Remove Throttling");
    invoke("remove_interface_throttling_wrapper", { interface: selectedInterface }).then((res) => {
      console.log(res);
    }).catch((err) => {
      console.log(err);
    });
  }

  useEffect (() => {
  const fetchData = async () => {
    invoke("get_interfaces_wrapper").then((res) => {
      console.log(res);
      const data = JSON.parse(res);
      console.log(data);
      setOptions(data.map((option) => ({ value: option.interface_name, label: option.interface_name })));
    }).catch((err) => {
      console.log(err);
    });
  };
    fetchData();
  }, []);

  return (
    <div className={styles.controlPanel}>
      <div className={styles.controlRow}>
        <button className={styles.controlButton} onClick={handleThInt} >Throttle Interface in (kBs)</button>
        <Dropdown options={options} prompt="Select an Interface" value={selectedInterface} onChange={setSelectedInterface} />
        <input type="text" className={styles.controlInput} placeholder='rate in kBs' onChange={(e) => setinThRate(e.target.value)}/>
        <button className={styles.controlRmInput} onClick={handleRmTh} >Remove Throttling</button>
      </div>
      <div className={styles.controlRow}>
        <button className={styles.controlButton} onClick={handleThApp}>Launch a Throttled App (kBs)</button>
        <input type="text" className={styles.controlInput} placeholder='up rate in kBs' onChange={(e) => setAppThUprate(e.target.value)}/>
        <input type="text" className={styles.controlInput} placeholder='down rate in kBs' onChange={(e) => setAppThDownrate(e.target.value)}/>
        <input type="text" className={styles.controlInput} placeholder='app name' onChange={(e) => setApp(e.target.value)}/>
      </div>
      <div className={styles.controlRow}>
        <button className={styles.controlButton} onClick={handleUpdateInterval}>Set Update Interval (ms)</button>
        <input type="text" className={styles.controlInput} placeholder='ip'/>
        <input type="text" className={styles.controlInput} placeholder='rate in kBs'/>
      </div>
    </div>
  );
};

export default ControlPanel;