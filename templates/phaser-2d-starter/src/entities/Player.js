/**
 * Player entity
 * Handles player movement, physics, and interactions
 */

export default class Player {
    /**
     * @param {Phaser.Scene} scene - The scene this player belongs to
     * @param {number} x - Initial X position
     * @param {number} y - Initial Y position
     */
    constructor(scene, x, y) {
        this.scene = scene;
        
        // Create the player sprite
        this.sprite = scene.physics.add.sprite(x, y, 'player');
        
        // Configure physics properties
        this.sprite.setBounce(0.1);
        this.sprite.setCollideWorldBounds(true);
        
        // Movement speed
        this.speed = 200;
        
        // Setup collision with platforms
        scene.physics.add.collider(this.sprite, scene.platforms);
    }
    
    /**
     * Update player state based on input
     * @param {object} cursors - Phaser cursor keys object
     */
    update(cursors) {
        // Reset velocity
        this.sprite.setVelocity(0);
        
        // Horizontal movement
        if (cursors.left.isDown) {
            this.sprite.setVelocityX(-this.speed);
        } else if (cursors.right.isDown) {
            this.sprite.setVelocityX(this.speed);
        }
        
        // Vertical movement
        if (cursors.up.isDown) {
            this.sprite.setVelocityY(-this.speed);
        } else if (cursors.down.isDown) {
            this.sprite.setVelocityY(this.speed);
        }
    }
    
    /**
     * Get player position
     * @returns {object} Position with x and y properties
     */
    getPosition() {
        return {
            x: this.sprite.x,
            y: this.sprite.y
        };
    }
    
    /**
     * Set player position
     * @param {number} x - X coordinate
     * @param {number} y - Y coordinate
     */
    setPosition(x, y) {
        this.sprite.setPosition(x, y);
    }
}
