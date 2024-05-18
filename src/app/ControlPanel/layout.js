import React from 'react';
import styles from './layout.module.css';

const ControlPanel = ({ onClose }) => {
  return (
    <div className={styles.controlPanel}>
      <button onClick={onClose} className={styles.closeButton}>Close</button>
      <div className={styles.controlRow}>
        <button className={styles.controlButton}>Button 1</button>
        <input type="text" className={styles.controlInput} />
      </div>
      <div className={styles.controlRow}>
        <button className={styles.controlButton}>Button 2</button>
        <input type="text" className={styles.controlInput} />
      </div>
      <div className={styles.controlRow}>
        <button className={styles.controlButton}>Button 3</button>
        <input type="text" className={styles.controlInput} />
      </div>
    </div>
  );
};

export default ControlPanel;
